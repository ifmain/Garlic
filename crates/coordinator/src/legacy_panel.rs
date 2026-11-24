use crate::{
    accounts::now,
    http::{bearer, blocking, check_lease, header, Shared},
};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use garlic_relay_data::control::Role;
use reqwest::Url;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha224};
use std::{
    collections::{HashMap, HashSet},
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};

const UNAVAILABLE: StatusCode = StatusCode::SERVICE_UNAVAILABLE;
const BAD_PANEL_REPLY: StatusCode = StatusCode::BAD_GATEWAY;
const PANEL_CONFLICT: StatusCode = StatusCode::CONFLICT;

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", tag = "action", deny_unknown_fields)]
pub(crate) enum Action {
    Create {
        email: String,
        expiry_ms: i64,
        inbound_ids: Vec<u64>,
    },
    Extend {
        email: String,
        days: u16,
    },
    Subscription {
        email: String,
    },
}

struct Panel {
    api: Url,
    subscription: Url,
    w_node_id: u64,
    client: reqwest::Client,
    token: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Inbound {
    id: u64,
    enable: bool,
    node_id: Option<u64>,
    port: u16,
    protocol: String,
}

#[derive(Deserialize, Serialize)]
struct Traffic {
    down: u64,
    up: u64,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LegacyUser {
    email: String,
    enable: bool,
    expiry_time: i64,
    #[serde(rename = "totalGB")]
    total_gb: i64,
    inbound_ids: Vec<u64>,
    traffic: Option<Traffic>,
}

#[derive(Deserialize)]
struct UserPage {
    items: Vec<LegacyUser>,
    total: usize,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkerInbound {
    id: u64,
    node_id: u64,
    protocol: String,
    enable: bool,
    total: Option<u64>,
    expiry_time: Option<u64>,
    #[serde(default)]
    settings: Value,
    #[serde(default)]
    client_stats: Value,
}

#[derive(Serialize)]
struct WorkerAccount {
    sha224: String,
    expiry_ms: i64,
}

impl WorkerInbound {
    fn accounts(self, inbound_id: u64, node_id: u64) -> Result<Vec<WorkerAccount>, StatusCode> {
        if self.id != inbound_id || self.node_id != node_id || self.protocol != "trojan" {
            return Err(BAD_PANEL_REPLY);
        }
        if !self.enable {
            return Ok(Vec::new());
        }
        if self.total != Some(0) || self.expiry_time != Some(0) {
            return Err(BAD_PANEL_REPLY);
        }
        let settings = match self.settings {
            Value::String(source) => serde_json::from_str(&source).map_err(|_| BAD_PANEL_REPLY)?,
            object => object,
        };
        let clients = settings["clients"].as_array().ok_or(BAD_PANEL_REPLY)?;
        let mut traffic = HashMap::new();
        if let Some(entries) = self.client_stats.as_array() {
            for entry in entries {
                if let Some(email) = entry["email"].as_str() {
                    traffic.entry(email).or_insert(entry);
                }
            }
        }

        let mut hashes = HashSet::new();
        let mut accounts = Vec::new();
        for client in clients {
            if !client["enable"].as_bool().ok_or(BAD_PANEL_REPLY)? {
                continue;
            }
            let email = client["email"].as_str().ok_or(BAD_PANEL_REPLY)?;
            let stat = traffic.get(email).ok_or(BAD_PANEL_REPLY)?;
            if !stat["enable"].as_bool().ok_or(BAD_PANEL_REPLY)?
                || ["limitIp", "totalGB", "reset"].iter().any(|key| {
                    client
                        .get(key)
                        .is_some_and(|value| value.as_i64() != Some(0))
                })
                || stat["total"].as_u64() != Some(0)
            {
                continue;
            }
            let client_expiry = client["expiryTime"].as_i64().ok_or(BAD_PANEL_REPLY)?;
            let stat_expiry = stat["expiryTime"].as_i64().ok_or(BAD_PANEL_REPLY)?;
            if client_expiry < 0 || stat_expiry < 0 {
                continue;
            }
            let expiry_ms = [client_expiry, stat_expiry]
                .into_iter()
                .filter(|value| *value > 0)
                .min()
                .unwrap_or(0);
            let password = client["password"]
                .as_str()
                .filter(|value| !value.is_empty())
                .ok_or(BAD_PANEL_REPLY)?;
            let hash = format!("{:x}", Sha224::digest(password.as_bytes()));
            if !hashes.insert(hash.clone()) {
                return Err(BAD_PANEL_REPLY);
            }
            accounts.push(WorkerAccount {
                sha224: hash,
                expiry_ms,
            });
        }
        Ok(accounts)
    }
}

pub(crate) async fn worker_directory(
    State(app): State<Shared>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let token = bearer(&headers).map_err(|failure| failure.0)?.to_owned();
    let node_id = blocking(move || {
        crate::network::identity(&app, &token)
            .map(Some)
            .or_else(|error| app.accounts.user(&token).map(|_| None).map_err(|_| error))
    })
    .await
    .map_err(|failure| failure.0)?;
    let authorized = env::var("GARLIC_XUI_W_IDENTITY").map_err(|_| StatusCode::NOT_FOUND)?;
    if node_id.as_deref() != Some(authorized.as_str()) {
        return Err(StatusCode::FORBIDDEN);
    }

    let panel = Panel::local().await?;
    let inbound_id = env::var("GARLIC_XUI_TROJAN_INBOUND")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|id| *id > 0)
        .ok_or(UNAVAILABLE)?;
    let inbound = panel
        .call(&["inbounds", "get", &inbound_id.to_string()], &[], None)
        .await?;
    let accounts = serde_json::from_value::<WorkerInbound>(inbound)
        .map_err(|_| BAD_PANEL_REPLY)?
        .accounts(inbound_id, panel.w_node_id)?;
    let body = Json(json!({"accounts": accounts, "valid_until_ms": now() * 1000 + 120_000}));
    Ok(([(axum::http::header::CACHE_CONTROL, "no-store")], body).into_response())
}

fn directory(value: &str) -> Result<Url, StatusCode> {
    let url = Url::parse(value).map_err(|_| UNAVAILABLE)?;
    if url.host().is_none()
        || !url.path().ends_with('/')
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(UNAVAILABLE);
    }
    Ok(url)
}

impl Panel {
    async fn local() -> Result<Self, StatusCode> {
        let api = directory(&env::var("GARLIC_XUI_API_URL").map_err(|_| StatusCode::NOT_FOUND)?)?;
        let subscription = directory(&env::var("GARLIC_XUI_SUB_URL").map_err(|_| UNAVAILABLE)?)?;
        let w_node_id = env::var("GARLIC_XUI_W_NODE_ID")
            .ok()
            .and_then(|value| value.parse().ok())
            .filter(|id| *id > 0)
            .ok_or(UNAVAILABLE)?;
        let host = api.host_str().ok_or(UNAVAILABLE)?;
        let numeric = host.trim_matches(['[', ']']).parse::<IpAddr>().ok();
        let is_local_http =
            host == "localhost" || matches!(numeric, Some(IpAddr::V4(ip)) if ip.is_loopback());
        if (api.scheme() != "https" && !(api.scheme() == "http" && is_local_http))
            || subscription.scheme() != "https"
            || (numeric.is_some() && !is_local_http)
        {
            return Err(UNAVAILABLE);
        }
        let port = api.port_or_known_default().ok_or(UNAVAILABLE)?;
        // Public panel names can have an HTTPS certificate, but the API connection is local.
        let mut builder = reqwest::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .resolve(host, SocketAddr::from((Ipv4Addr::LOCALHOST, port)))
            .timeout(Duration::from_secs(8));
        if let Some(path) = env::var_os("GARLIC_CA_CERT") {
            let ca = tokio::fs::read(path).await.map_err(|_| UNAVAILABLE)?;
            for certificate in
                reqwest::Certificate::from_pem_bundle(&ca).map_err(|_| UNAVAILABLE)?
            {
                builder = builder.add_root_certificate(certificate);
            }
        }
        let client = builder.build().map_err(|_| UNAVAILABLE)?;
        let token =
            tokio::fs::read_to_string(env::var_os("GARLIC_XUI_TOKEN_FILE").ok_or(UNAVAILABLE)?)
                .await
                .map_err(|_| UNAVAILABLE)?;
        let token = token.trim().to_owned();
        if token.is_empty() {
            return Err(UNAVAILABLE);
        }
        Ok(Self {
            api,
            subscription,
            w_node_id,
            client,
            token,
        })
    }

    async fn call(
        &self,
        path: &[&str],
        query: &[(&str, &str)],
        body: Option<Value>,
    ) -> Result<Value, StatusCode> {
        let mut url = self.api.clone();
        url.path_segments_mut()
            .map_err(|_| BAD_PANEL_REPLY)?
            .pop_if_empty()
            .extend(path);
        let request = if let Some(body) = body {
            self.client.post(url).json(&body)
        } else {
            self.client.get(url)
        };
        let mut response = request
            .bearer_auth(&self.token)
            .query(query)
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|_| BAD_PANEL_REPLY)?;
        let mut data = Vec::new();
        while let Some(part) = response.chunk().await.map_err(|_| BAD_PANEL_REPLY)? {
            if data.len() + part.len() > 8_388_608 {
                return Err(BAD_PANEL_REPLY);
            }
            data.extend_from_slice(&part);
        }
        let reply: Value = serde_json::from_slice(&data).map_err(|_| BAD_PANEL_REPLY)?;
        if reply["success"] != true {
            return Err(PANEL_CONFLICT);
        }
        Ok(reply["obj"].clone())
    }

    async fn get<T: DeserializeOwned>(
        &self,
        path: &[&str],
        query: &[(&str, &str)],
    ) -> Result<T, StatusCode> {
        serde_json::from_value(self.call(path, query, None).await?).map_err(|_| PANEL_CONFLICT)
    }

    async fn inbounds(&self) -> Result<Vec<Inbound>, StatusCode> {
        self.get(&["inbounds", "options"], &[]).await
    }

    async fn users(&self) -> Result<Vec<LegacyUser>, StatusCode> {
        const PAGE_SIZE: usize = 100;
        let mut page = 1;
        let mut users = Vec::new();
        loop {
            let number = page.to_string();
            let request = [
                ("page", number.as_str()),
                ("pageSize", "100"),
                ("sort", "email"),
                ("order", "asc"),
            ];
            let result: UserPage = self.get(&["clients", "list", "paged"], &request).await?;
            let size = result.items.len();
            users.extend(result.items);
            if size < PAGE_SIZE || users.len() >= result.total {
                return Ok(users);
            }
            page += 1;
        }
    }

    async fn create(
        &self,
        email: &str,
        expiry: i64,
        requested: &[u64],
    ) -> Result<Value, StatusCode> {
        let options = self.inbounds().await?;
        let [id] = requested else {
            return Err(StatusCode::BAD_REQUEST);
        };
        if !options.iter().any(|entry| {
            entry.id == *id
                && entry.enable
                && entry.protocol == "trojan"
                && entry.node_id == Some(self.w_node_id)
        }) {
            return Err(StatusCode::BAD_REQUEST);
        }
        let payload = json!({
            "client": {
                "email":email,
                "enable":true,
                "expiryTime":expiry,
                "password":uuid::Uuid::new_v4().simple().to_string(),
                "security":"auto",
                "subId":uuid::Uuid::new_v4().simple().to_string()
            },
            "inboundIds":[id]
        });
        let result = self.call(&["clients", "add"], &[], Some(payload)).await?;
        Ok(json!({"ok":true,"nodePending":result["nodePending"] == true}))
    }

    async fn subscription(&self, email: &str) -> Result<Value, StatusCode> {
        let client = self.call(&["clients", "get", email], &[], None).await?;
        let sub_id = client["client"]["subId"]
            .as_str()
            .filter(|id| !id.is_empty() && *id != "." && *id != "..")
            .ok_or(PANEL_CONFLICT)?;
        let mut link = self.subscription.clone();
        link.path_segments_mut()
            .map_err(|_| PANEL_CONFLICT)?
            .pop_if_empty()
            .push(sub_id);
        Ok(json!({"ok":true,"url":link.as_str()}))
    }

    async fn extend(&self, email: &str, days: u16) -> Result<Value, StatusCode> {
        let result = self
            .call(
                &["clients", "bulkAdjust"],
                &[],
                Some(json!({"addDays":days,"emails":[email]})),
            )
            .await?;
        if result["adjusted"].as_u64() != Some(1) {
            return Err(PANEL_CONFLICT);
        }
        Ok(json!({"ok":true}))
    }
}

async fn operator(app: Shared, headers: &HeaderMap) -> Result<(), Response> {
    let token = bearer(headers).map_err(IntoResponse::into_response)?;
    let account_token = token.to_owned();
    let accounts = app.clone();
    let (user, _) = blocking(move || accounts.accounts.user(&account_token))
        .await
        .map_err(IntoResponse::into_response)?;
    check_lease(
        app,
        token,
        &user,
        header(headers, "x-garlic-device-token").unwrap_or_default(),
    )
    .await
    .map_err(IntoResponse::into_response)?;
    if user.role != Role::Admin {
        return Err(StatusCode::FORBIDDEN.into_response());
    }
    Ok(())
}

pub(crate) async fn list(
    State(app): State<Shared>,
    headers: HeaderMap,
) -> Result<Json<Value>, Response> {
    operator(app, &headers).await?;
    let panel = Panel::local().await.map_err(IntoResponse::into_response)?;
    let routes = panel
        .inbounds()
        .await
        .map_err(IntoResponse::into_response)?;
    let users = panel.users().await.map_err(IntoResponse::into_response)?;
    Ok(Json(
        json!({"routes":routes,"users":users,"worker":panel.w_node_id}),
    ))
}

pub(crate) async fn change(
    State(app): State<Shared>,
    headers: HeaderMap,
    Json(action): Json<Action>,
) -> Result<Json<Value>, Response> {
    operator(app, &headers).await?;
    let panel = Panel::local().await.map_err(IntoResponse::into_response)?;
    let email = match &action {
        Action::Create { email, .. }
        | Action::Extend { email, .. }
        | Action::Subscription { email } => email,
    };
    if email.is_empty() || email == "." || email == ".." {
        return Err(StatusCode::BAD_REQUEST.into_response());
    }
    let reply = match action {
        Action::Create {
            email,
            expiry_ms,
            inbound_ids,
        } => panel.create(&email, expiry_ms, &inbound_ids).await,
        Action::Extend { email, days } => panel.extend(&email, days).await,
        Action::Subscription { email } => panel.subscription(&email).await,
    };
    Ok(Json(reply.map_err(IntoResponse::into_response)?))
}
