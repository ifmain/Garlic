use crate::{
    accounts::now,
    http::{bearer, blocking, header, Failure, Shared},
    hub::Outbound,
};
use axum::{
    extract::{
        ws::{Message as Ws, WebSocket},
        State, WebSocketUpgrade,
    },
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use garlic_relay_data::{
    control::{Hello, Message},
    envelope, Direction, Frame,
};
use garlic_relay_session::{
    auth::verify_node,
    coordinator::{handshake_error, NodeSession},
    SessionError,
};
use std::{num::NonZeroU32, time::Duration};
use tokio::sync::mpsc;
use uuid::Uuid;

pub async fn upgrade(
    State(app): State<Shared>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Result<Response, Failure> {
    let token = bearer(&headers)?.to_owned();
    if header(&headers, "x-garlic-exchange-version") != Some("2") {
        return Err(Failure(
            StatusCode::BAD_REQUEST,
            "unsupported relay version".into(),
        ));
    }
    let auth = app.clone();
    let (node, label) = blocking(move || {
        let node = verify_node(
            &auth.accounts.accounts,
            &auth.accounts.keys,
            &auth.accounts.owner,
            &token,
            now(),
        )?;
        let label = auth.accounts.node_label(&node.node_id)?;
        Ok((node, label))
    })
    .await?;
    let authority = node.authority.clone();
    let mut response = ws
        .max_message_size(65_536)
        .max_frame_size(65_536)
        .on_upgrade(move |socket| async move {
            if let Err(e) = session(socket, app, node, label).await {
                eprintln!("Worker relay ended: {e}");
            }
        })
        .into_response();
    let out = response.headers_mut();
    out.insert("x-garlic-exchange-version", HeaderValue::from_static("2"));
    if header(&headers, "x-garlic-relay-drain-ack") == Some("1") {
        out.insert("x-garlic-relay-drain-ack", HeaderValue::from_static("1"));
    }
    out.insert(
        "x-garlic-owner-id",
        authority.owner_id.to_string().parse().unwrap(),
    );
    out.insert("x-garlic-owner-epoch", authority.epoch.into());
    Ok(response)
}

pub struct Encoder {
    id: NonZeroU32,
}
impl Default for Encoder {
    fn default() -> Self {
        Self {
            id: NonZeroU32::MIN,
        }
    }
}
impl Encoder {
    pub async fn send(
        &mut self,
        socket: &mut WebSocket,
        kind: u8,
        body: &[u8],
    ) -> Result<(), SessionError> {
        let mut message = Vec::with_capacity(body.len() + 1);
        message.push(kind);
        message.extend_from_slice(body);
        let id = self.id;
        self.id = NonZeroU32::new(id.get().wrapping_add(1)).unwrap_or(NonZeroU32::MIN);
        for record in
            envelope::fragments(id, &message).map_err(|_| "invalid relay message length")?
        {
            socket.send(Ws::Binary(record.into())).await?;
        }
        Ok(())
    }
    async fn control(
        &mut self,
        socket: &mut WebSocket,
        control: &Message,
    ) -> Result<(), SessionError> {
        self.send(socket, 1, &serde_json::to_vec(control)?).await
    }
}
async fn session(
    mut ws: WebSocket,
    app: Shared,
    node: NodeSession,
    label: String,
) -> Result<(), SessionError> {
    let first = tokio::time::timeout(Duration::from_secs(15), ws.recv())
        .await?
        .ok_or("worker did not announce itself")??;
    let Ws::Text(text) = first else {
        return Err("expected worker hello".into());
    };
    let hello: Hello = serde_json::from_str(text.as_str())?;
    if let Some(code) = handshake_error(&hello, &node.node_id) {
        let error: garlic_relay_data::control::ProxyError = code.into();
        let msg = serde_json::json!({"type":"rejected","protocol_version":1,"error":error});
        ws.send(Ws::Text(msg.to_string().into())).await?;
        return Ok(());
    }
    let id = hello.worker_id;
    let (tx, mut outbound) = mpsc::channel(128);
    if !app.hub.register(hello, label, tx) {
        return Err("duplicate worker identifier".into());
    }
    struct Registration(Shared, Uuid);
    impl Drop for Registration {
        fn drop(&mut self) {
            self.0.hub.remove_worker(self.1);
        }
    }
    let _registered = Registration(app.clone(), id);
    let ready = Message::Ready {
        protocol_version: 1,
        worker_id: id,
        authority: node.authority,
    };
    ws.send(Ws::Text(serde_json::to_string(&ready)?.into()))
        .await?;
    let mut encoder = Encoder::default();
    let mut decoder = envelope::Decoder::default();
    let mut sync = tokio::time::interval(Duration::from_secs(5));
    sync.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            _=sync.tick()=>{
                let state=app.clone();
                let snapshot=tokio::task::spawn_blocking(move||state.accounts.snapshot()).await??;
                encoder.control(&mut ws,&Message::UserSync{snapshot}).await?;
            }
            outgoing=outbound.recv()=>match outgoing {
                Some(Outbound::Control(control))=>encoder.control(&mut ws,&control).await?,
                Some(Outbound::Data(flow_id,payload))=>{
                    let data=Frame{direction:Direction::One,flow_id:*flow_id.as_bytes(),payload:&payload}.encode().map_err(|_|"invalid relay payload")?;
                    encoder.send(&mut ws,0,&data).await?;
                }
                None=>break,
            },
            incoming=ws.recv()=>match incoming.transpose()?{
                None|Some(Ws::Close(_))=>break,
                Some(Ws::Ping(payload))=>ws.send(Ws::Pong(payload)).await?,
                Some(Ws::Pong(_))=>{},
                Some(Ws::Binary(record))=>{
                    let Some(logical)=decoder.receive(&record).map_err(|_|"invalid relay record")? else{continue;};
                    let (&kind,body)=logical.split_first().ok_or("empty relay record")?;
                    match kind {
                        0=>{let frame=garlic_relay_data::decode(body).map_err(|_|"invalid relay frame")?;
                            if frame.direction!=Direction::Two{return Err("invalid exit frame direction".into());}
                            app.hub.downstream(id,Uuid::from_bytes(frame.flow_id),frame.payload.to_vec());}
                        1=>match serde_json::from_slice(body)?{
                            Message::Opened{request_id,error}=>app.hub.opened(id,request_id,error),
                            Message::Drain=>{app.hub.drain(id);encoder.control(&mut ws,&Message::Drained).await?;}
                            _=>return Err("unexpected control message from exit".into()),
                        },
                        2=>encoder.send(&mut ws,3,body).await?,
                        3=>{},
                        4=>break,
                        _=>return Err("unrecognized relay message".into()),
                    }
                }
                _=>return Err("unexpected WebSocket message".into()),
            }
        }
    }
    Ok(())
}
