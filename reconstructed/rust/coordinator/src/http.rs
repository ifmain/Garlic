use crate::accounts::{now, AccountsService, SubscriptionExpired};
use crate::devices::{DeviceError, DeviceStore};
use axum::{
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use garlic_relay_data::control::{Subscription, User};
use garlic_relay_session::{
    auth::verify_node,
    identity::{ChallengeRequest, Proof},
    SessionError,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{io, sync::Arc};

pub struct App {
    pub update_dir: Option<std::path::PathBuf>,
    pub service_name: String,
    pub jobs: crate::queue::store::Jobs,
    pub authority: Option<crate::upstream::AccountAuthority>,
    pub network: crate::network::Network,
    pub(crate) modules: crate::exchanges::modules::Registry,
    pub exchanges: crate::exchanges::Exchanges,
    pub hub: crate::hub::Hub,
    pub accounts: AccountsService,
    pub devices: DeviceStore,
    pub require_device: bool,
}
pub type Shared = Arc<App>;
pub async fn authority_headers(
    State(app): State<Shared>,
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    const ID: &str = "x-garlic-owner-id";
    const EPOCH: &str = "x-garlic-owner-epoch";
    // Use fixed stage names only. Request paths, credentials and headers may contain user data.
    let stage = match req.uri().path() {
        "/v1/auth/info" => Some("discovery"),
        "/v1/auth/login" => Some("login"),
        "/v1/auth/me" => Some("profile"),
        "/v1/network" => Some("directory"),
        "/v1/auth/device/claim" => Some("device_claim"),
        "/v1/auth/device/check" => Some("device_check"),
        "/v1/auth/device/renew" => Some("device_renew"),
        "/v1/infinity/streams" => Some("stream_open"),
        p if p.starts_with("/v1/exchange/") => Some("exchange_open"),
        _ => None,
    };
    let h = req.headers();
    let owner = &app.accounts.owner;
    let matches = (!h.contains_key(ID) && !h.contains_key(EPOCH))
        || (header(h, ID).and_then(|v| uuid::Uuid::parse_str(v).ok()) == Some(owner.owner_id)
            && header(h, EPOCH).and_then(|v| v.parse().ok()) == Some(owner.epoch));
    let mut response = if matches {
        next.run(req).await
    } else {
        (
            StatusCode::PRECONDITION_FAILED,
            Json(json!({"code":"authority_mismatch","error":"authority or epoch mismatch"})),
        )
            .into_response()
    };
    if let Some(stage) = stage {
        let status = response.status();
        if status.is_client_error() || status.is_server_error() {
            if let Some(reason) = response.extensions().get::<garlic_relay_data::control::ErrorCode>() {
                eprintln!("connection_failure stage={stage} status={} reason={reason:?}", status.as_u16());
            } else {
                eprintln!("connection_failure stage={stage} status={}", status.as_u16());
            }
        }
    }
    let headers = response.headers_mut();
    headers.insert(
        ID,
        owner
            .owner_id
            .to_string()
            .parse()
            .expect("UUID is an HTTP header value"),
    );
    headers.insert(EPOCH, owner.epoch.into());
    response
}
pub struct Failure(pub StatusCode, pub String);
impl IntoResponse for Failure {
    fn into_response(self) -> Response {
        (self.0, Json(json!({"error":self.1}))).into_response()
    }
}
impl From<SessionError> for Failure {
    fn from(e: SessionError) -> Self {
        if let Some(err) = e.downcast_ref::<crate::queue::model::QueueError>() {
            use crate::queue::model::QueueError::*;
            return Self(
                match err {
                    Missing => StatusCode::NOT_FOUND,
                    Conflict => StatusCode::CONFLICT,
                    Invalid => StatusCode::BAD_REQUEST,
                    Unsupported => StatusCode::UPGRADE_REQUIRED,
                    Forbidden => StatusCode::FORBIDDEN,
                    Capacity => StatusCode::TOO_MANY_REQUESTS,
                },
                err.to_string(),
            );
        }
        if e.is::<crate::accounts::admin::AdminRequired>() {
            return Self(StatusCode::FORBIDDEN, e.to_string());
        }
        if e.downcast_ref::<io::Error>()
            .is_some_and(|err| err.kind() == io::ErrorKind::InvalidInput)
        {
            return Self(StatusCode::BAD_REQUEST, "authentication Invalid".into());
        }
        if let Some(err) = e.downcast_ref::<DeviceError>() {
            return Self(
                match err {
                    DeviceError::Invalid => StatusCode::BAD_REQUEST,
                    DeviceError::InUse => StatusCode::CONFLICT,
                },
                err.to_string(),
            );
        }
        if e.is::<SubscriptionExpired>() {
            return Self(StatusCode::PAYMENT_REQUIRED, e.to_string());
        }
        if e.downcast_ref::<io::Error>()
            .is_some_and(|err| err.kind() == io::ErrorKind::PermissionDenied)
        {
            return Self(
                StatusCode::UNAUTHORIZED,
                "authentication Unauthorized".into(),
            );
        }
        Self(
            StatusCode::INTERNAL_SERVER_ERROR,
            "account service error".into(),
        )
    }
}
type Reply = Result<Json<Value>, Failure>;
pub fn header<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name)?.to_str().ok()
}
pub fn bearer(headers: &HeaderMap) -> Result<&str, Failure> {
    header(headers, "authorization")
        .and_then(|h| h.strip_prefix("Bearer "))
        .filter(|s| !s.is_empty() && s.len() <= 1024)
        .ok_or_else(|| {
            Failure(
                StatusCode::UNAUTHORIZED,
                "authentication Unauthorized".into(),
            )
        })
}
pub async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> Result<T, SessionError> + Send + 'static,
) -> Result<T, Failure> {
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|_| {
            Failure(
                StatusCode::INTERNAL_SERVER_ERROR,
                "account service interrupted".into(),
            )
        })?
        .map_err(Into::into)
}

pub fn routes() -> Router<Shared> {
    Router::new()
        .route("/v1/infinity/streams", post(crate::streams::open))
        .route(
            "/v1/infinity/streams/{id}",
            axum::routing::delete(crate::streams::close),
        )
        .route("/v1/exchange/{id}", get(crate::exchanges::upgrade))
        .route("/v1/infinity/relay", get(crate::worker::upgrade))
        .route("/health", get(health))
        .route("/v1/auth/info", get(info))
        .route("/v1/auth/updates/{name}", get(crate::updates::get))
        .route("/v1/network", get(crate::network::get))
        .route("/v1/infra/network/register", post(crate::network::register))
        .route(
            "/v1/infra/network/observation",
            post(crate::network::observation),
        )
        .route("/v1/admin/network/policy", post(crate::network::set_policy))
        .route(
            "/v1/admin/legacy",
            get(crate::legacy_panel::list).post(crate::legacy_panel::change),
        )
        .route("/v1/auth/login", post(login))
        .route("/v1/auth/me", get(me))
        .route("/v1/auth/users", get(users).post(manage))
        .route("/v1/auth/logout", post(logout))
        .route("/v1/auth/device/claim", post(claim))
        .route("/v1/auth/device/renew", post(renew))
        .route("/v1/auth/device/check", post(check))
        .route("/v1/auth/device/release", post(release))
        .route("/v1/infra/challenge", post(challenge))
        .route("/v1/infra/prove", post(prove))
        .route("/v1/infra/users/sync", get(snapshot))
        .route(
            "/v1/infra/legacy/accounts",
            get(crate::legacy_panel::worker_directory),
        )
        .layer(DefaultBodyLimit::max(65_536))
        .merge(crate::queue::routes())
        .merge(crate::queue::transfers::routes())
}
async fn health(State(app): State<Shared>, h: HeaderMap) -> Reply {
    let token = bearer(&h).ok().map(str::to_owned);
    let name = if let Some(token) = token {
        blocking(move || {
            Ok(
                if app.accounts.user(&token).is_ok()
                    || crate::network::identity(&app, &token).is_ok()
                {
                    app.service_name.clone()
                } else {
                    "api".into()
                },
            )
        })
        .await?
    } else {
        "api".into()
    };
    Ok(Json(json!({"healthy":true,"service":name})))
}

pub(crate) fn profile(user: &User) -> Value {
    let mut data = json!({"user_id":user.id,"username":user.username,"role":user.role,"generation":user.generation,
        "enabled":user.enabled,"revision":user.revision,"subscription":user.subscription});
    match user.subscription {
        Subscription::Unlimited => {
            data["status"] = json!("unlimited");
            data["subscription_display"] = json!("unlimited");
        }
        Subscription::ExpiresAt { expires_at } => {
            data["status"] = json!(if expires_at > now() {
                "active"
            } else {
                "expired"
            });
            if let Some(dt) = i64::try_from(expires_at)
                .ok()
                .and_then(|s| chrono::DateTime::from_timestamp(s, 0))
            {
                let utc = dt.format("%Y-%m-%dT%H:%M:%SZ").to_string();
                data["subscription_display"] =
                    json!(format!("{} +00:00 ({utc})", dt.format("%Y-%m-%d %H:%M:%S")));
                data["subscription_expires_utc"] = json!(utc);
            }
        }
    }
    data
}

async fn info(State(app): State<Shared>, headers: HeaderMap) -> Reply {
    let mut data = json!({"version":1,"authority":app.accounts.owner,"max_window":1048576,"device_leases":app.require_device});
    if let Ok(token) = bearer(&headers) {
        let token = token.to_owned();
        let local = app.clone();
        let (user, _) = blocking(move || local.accounts.user(&token)).await?;
        if user.role == garlic_relay_data::control::Role::Admin {
            data["device_leases"] = json!(false);
        }
        data["load_percent"] = json!(app.hub.load_percent());
        if let Some(path) = std::env::var_os("GARLIC_RECORD_PROFILE_FILE") {
            use tokio::io::AsyncReadExt;
            if let Ok(file) = tokio::fs::File::open(path).await {
                let mut bytes = Vec::new();
                if file.take(2049).read_to_end(&mut bytes).await.is_ok() && bytes.len() <= 2048 {
                    if let Ok(profile) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                        data["record_profile"] = profile;
                    }
                }
            }
        }
        data["authentication"] = json!("separate");
        data["server_version"] = json!("0.9.2");
        let versions = app.modules.versions();
        // Deployed clients use this scalar as a discovery-format marker.
        // The list and capability determine the protocols accepted at upgrade.
        data["exchange_version"] = json!(1);
        data["exchange_versions"] = json!(versions.iter().filter(|&&n| n <= 3).collect::<Vec<_>>());
        let additional: Vec<_> = versions.iter().copied().filter(|v| *v >= 128).collect();
        if !additional.is_empty() { data["exchange_module_versions"] = json!(additional); }
        if versions.contains(&4) {
            data["exchange_extensions"] = json!([garlic_relay_data::gex4::CAPABILITY]);
        }
        data["exchange_shape_profile"] = json!(crate::exchanges::SHAPE_PROFILE);
    }
    Ok(Json(data))
}

#[derive(Deserialize)]
struct Login {
    username: String,
    password: String,
    #[serde(default)]
    network_session: bool,
}
async fn login(State(app): State<Shared>, Json(body): Json<Login>) -> Reply {
    if let Some(remote) = &app.authority {
        let mut grant = remote.request("v1/auth/login", "", Some(json!({
            "username":body.username,"password":body.password,"network_session":true
        }))).await?;
        let token = grant["token"].as_str().ok_or_else(|| Failure(StatusCode::BAD_GATEWAY, "invalid authority grant".into()))?.to_owned();
        // Wait for the authenticated snapshot so the first device claim can use the grant immediately.
        for _ in 0..80 {
            let local = app.clone();
            let auth = token.clone();
            if blocking(move || local.accounts.user(&auth)).await.is_ok() {
                grant["authority"] = json!(app.accounts.owner);
                return Ok(Json(grant));
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        return Err(Failure(StatusCode::SERVICE_UNAVAILABLE, "account synchronization pending".into()));
    }
    Ok(Json(
        blocking(move || {
            let grant = app.accounts.login_for_network(&body.username, &body.password, body.network_session)?;
            let (user, _) = app.accounts.user(&grant.token)?;
            let mut out = serde_json::to_value(grant)?;
            out["profile"] = profile(&user);
            Ok(out)
        })
        .await?,
    ))
}
async fn me(State(app): State<Shared>, headers: HeaderMap) -> Reply {
    let token = bearer(&headers)?.to_owned();
    let (user, session) = blocking(move || app.accounts.user(&token)).await?;
    let user = profile(&user);
    Ok(Json(
        json!({"principal":session.principal,"status":user["status"],"subscription":user["subscription"],
        "subscription_display":user["subscription_display"],"subscription_expires_utc":user["subscription_expires_utc"],"user":user}),
    ))
}
async fn logout(State(app): State<Shared>, headers: HeaderMap) -> Reply {
    let token = bearer(&headers)?.to_owned();
    if let Some(remote) = &app.authority {
        let local = app.clone();
        let auth = token.clone();
        if let Ok((_, session)) = blocking(move || local.accounts.user(&auth)).await {
            if session.owner.owner_id.is_nil() && token.len() == 46 {
                remote.request("v1/auth/logout", &token, Some(json!({}))).await?;
            }
        }
    }
    blocking(move || app.accounts.logout(&token)).await?;
    Ok(Json(json!({"ok":true})))
}
async fn users(State(app): State<Shared>, headers: HeaderMap) -> Reply {
    let token = bearer(&headers)?.to_owned();
    if let Some(remote) = &app.authority {
        return Ok(Json(remote.request("v1/auth/users", &token, None).await?));
    }
    Ok(Json(json!(
        blocking(move || app.accounts.admin_users(&token)).await?
    )))
}
async fn manage(
    State(app): State<Shared>,
    headers: HeaderMap,
    Json(body): Json<crate::accounts::admin::Manage>,
) -> Reply {
    let token = bearer(&headers)?.to_owned();
    if let Some(remote) = &app.authority {
        return Ok(Json(
            remote
                .request("v1/auth/users", &token, Some(json!(body)))
                .await?,
        ));
    }
    Ok(Json(
        blocking(move || app.accounts.admin_manage(&token, body)).await?,
    ))
}

#[derive(Deserialize, serde::Serialize, Default)]
struct DeviceRequest {
    #[serde(default)]
    device_id: String,
    #[serde(default)]
    takeover: bool,
    #[serde(default)]
    token: String,
}
async fn claim(
    State(app): State<Shared>,
    headers: HeaderMap,
    Json(body): Json<DeviceRequest>,
) -> Reply {
    let token = bearer(&headers)?.to_owned();
    let local = app.clone();
    let auth = token.clone();
    let (user, _) = blocking(move || local.accounts.user(&auth)).await?;
    if user.role == garlic_relay_data::control::Role::Admin {
        let id = uuid::Uuid::parse_str(&body.device_id).ok().filter(|id| !id.is_nil())
            .ok_or_else(|| Failure(StatusCode::BAD_REQUEST, "device_authorization_failed".into()))?;
        return Ok(Json(json!({"device_id":id,"generation":1,"expires_at":now()+60,"token":crate::accounts::random_secret("")?})));
    }
    if let Some(remote) = &app.authority {
        return Ok(Json(
            remote
                .request("v1/auth/device/claim", &token, Some(json!(body)))
                .await?,
        ));
    }
    let lease = blocking(move || {
        app.devices.claim(user.id, &body.device_id, body.takeover)
    })
    .await?;
    Ok(Json(json!(lease)))
}
pub async fn check_lease(
    app: Shared,
    token: &str,
    user: &User,
    device: &str,
) -> Result<(), Failure> {
    if !app.require_device || user.role == garlic_relay_data::control::Role::Admin {
        return Ok(());
    }
    if let Some(remote) = &app.authority {
        remote
            .request("v1/auth/device/check", token, Some(json!({"token":device})))
            .await?;
    } else {
        let id = user.id;
        let device = device.to_owned();
        blocking(move || app.devices.valid(id, &device)).await?;
    }
    Ok(())
}
async fn check(
    State(app): State<Shared>,
    headers: HeaderMap,
    Json(body): Json<DeviceRequest>,
) -> Reply {
    let token = bearer(&headers)?.to_owned();
    let local = app.clone();
    let auth = token.clone();
    let (user, _) = blocking(move || local.accounts.user(&auth)).await?;
    check_lease(app, &token, &user, &body.token).await?;
    Ok(Json(json!({"ok":true})))
}
async fn renew(
    State(app): State<Shared>,
    headers: HeaderMap,
    Json(body): Json<DeviceRequest>,
) -> Reply {
    update_device(app, headers, body, false).await
}
async fn release(
    State(app): State<Shared>,
    headers: HeaderMap,
    Json(body): Json<DeviceRequest>,
) -> Reply {
    update_device(app, headers, body, true).await
}
async fn update_device(
    app: Shared,
    headers: HeaderMap,
    body: DeviceRequest,
    release: bool,
) -> Reply {
    let token = bearer(&headers)?.to_owned();
    let local = app.clone();
    let auth = token.clone();
    let (user, _) = blocking(move || local.accounts.user(&auth)).await?;
    if user.role == garlic_relay_data::control::Role::Admin {
        return Ok(Json(json!({"ok":true})));
    }
    if let Some(remote) = &app.authority {
        return Ok(Json(
            remote
                .request(
                    if release {
                        "v1/auth/device/release"
                    } else {
                        "v1/auth/device/renew"
                    },
                    &token,
                    Some(json!(body)),
                )
                .await?,
        ));
    }
    blocking(move || {
        app.devices.update(user.id, &body.token, release)
    })
    .await?;
    Ok(Json(json!({"ok":true})))
}

async fn challenge(State(app): State<Shared>, Json(body): Json<ChallengeRequest>) -> Reply {
    Ok(Json(json!(
        blocking(move || app.accounts.challenge(body)).await?
    )))
}
async fn prove(State(app): State<Shared>, Json(body): Json<Proof>) -> Reply {
    Ok(Json(json!(
        blocking(move || app.accounts.prove(body)).await?
    )))
}
async fn snapshot(State(app): State<Shared>, headers: HeaderMap) -> Reply {
    let token = bearer(&headers)?.to_owned();
    Ok(Json(json!(
        blocking(move || {
            verify_node(
                &app.accounts.accounts,
                &app.accounts.keys,
                &app.accounts.owner,
                &token,
                now(),
            )?;
            app.accounts.snapshot()
        })
        .await?
    )))
}
