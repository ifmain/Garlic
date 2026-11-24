use super::{model::*, store::State as Jobs};
use crate::{
    files,
    http::{blocking, header, Failure, Shared},
};
use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, Path, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post, put},
    Json, Router,
};
use ring::digest::{digest, SHA256};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path as FsPath, PathBuf},
};
use uuid::Uuid;

const CHUNK: usize = 262144;
#[derive(Deserialize)]
pub struct Create {
    protocol_version: u32,
    session_id: Uuid,
    total_bytes: u64,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Part {
    length: u64,
    sha256: String,
    task_id: Uuid,
    received: bool,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Manifest {
    protocol_version: u32,
    session_id: Uuid,
    owner_id: Uuid,
    total_bytes: u64,
    uploaded: u64,
    received: u64,
    chunk_bytes: u64,
    parts: BTreeMap<u64, Part>,
    #[serde(flatten)]
    extra: Extra,
}
#[derive(Deserialize, Serialize)]
pub struct Work {
    pub session_id: Uuid,
    pub offset: u64,
}
#[derive(Deserialize)]
struct ReadLease {
    channel_id: Uuid,
    offset: u64,
    #[serde(flatten)]
    ids: LeaseRequest,
}
fn dir(state: &FsPath, id: Uuid) -> PathBuf {
    state.with_extension("transfers").join(id.to_string())
}
fn read(state: &FsPath, id: Uuid) -> Result<Manifest> {
    let f = dir(state, id).join("manifest.json");
    if !f.is_file() {
        return Err(QueueError::Missing.into());
    }
    let man: Manifest = files::read(&f)?;
    if man.protocol_version != 1 || man.session_id != id || man.chunk_bytes == 0 {
        return Err("invalid transfer manifest".into());
    }
    Ok(man)
}
fn owned(state: &FsPath, id: Uuid, user: Uuid) -> Result<Manifest> {
    let man = read(state, id)?;
    if man.owner_id != user {
        return Err(QueueError::Missing.into());
    }
    Ok(man)
}
fn save_bytes(path: &FsPath, b: &[u8]) -> Result<()> {
    let parent = path.parent().ok_or("missing transfer directory")?;
    let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
    tmp.write_all(b)?;
    tmp.as_file().sync_all()?;
    tmp.persist(path)?;
    #[cfg(unix)]
    fs::File::open(parent)?.sync_all()?;
    Ok(())
}
fn checksum(b: &[u8]) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(64);
    for v in digest(&SHA256, b).as_ref() {
        write!(&mut out, "{v:02x}").unwrap();
    }
    out
}
fn enqueue(s: &mut Jobs, path: &FsPath, m: &Manifest, offset: u64, part: &Part) -> Result<()> {
    s.submit(
        path,
        Task {
            task_id: part.task_id,
            operation: Operation::Transfer,
            input: serde_json::to_string(&Work {
                session_id: m.session_id,
                offset,
            })?,
            exchange: None,
            priority: Priority::Normal,
        },
        m.owner_id,
    )?;
    Ok(())
}
// A crash can happen between committing the manifest and writing the queue.
// The task UUID stored in the manifest makes this reconciliation idempotent.
pub fn recover(jobs: &super::store::Jobs, state: &FsPath) -> Result<()> {
    let path = state.with_extension("transfers");
    if !path.exists() {
        return Ok(());
    }
    jobs.with(|s, p, _| {
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            let Ok(id) = entry.file_name().to_string_lossy().parse::<Uuid>() else {
                continue;
            };
            if !entry.file_type()?.is_dir() || !entry.path().join("manifest.json").is_file() {
                continue;
            }
            let m = read(p, id)?;
            for (offset, part) in &m.parts {
                if !part.received
                    && s.disk
                        .records
                        .iter()
                        .all(|r| r.task.task_id != part.task_id)
                {
                    enqueue(s, p, &m, *offset, part)?;
                }
            }
        }
        Ok(())
    })
}
fn create(state: &FsPath, body: Create, user: Uuid) -> Result<Manifest> {
    if body.protocol_version != 1 {
        return Err(QueueError::Unsupported.into());
    }
    if body.total_bytes > 64 << 20 {
        return Err(QueueError::Capacity.into());
    }
    let folder = dir(state, body.session_id);
    let file = folder.join("manifest.json");
    if file.exists() {
        let m = owned(state, body.session_id, user)?;
        if m.total_bytes != body.total_bytes {
            return Err(QueueError::Conflict.into());
        }
        return Ok(m);
    }
    fs::create_dir_all(&folder)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&folder, fs::Permissions::from_mode(0o700))?;
    }
    let m = Manifest {
        protocol_version: 1,
        session_id: body.session_id,
        owner_id: user,
        total_bytes: body.total_bytes,
        uploaded: 0,
        received: 0,
        chunk_bytes: CHUNK as u64,
        parts: Default::default(),
        extra: Default::default(),
    };
    files::create(&file, &m)?;
    Ok(m)
}
fn upload(
    s: &mut Jobs,
    state: &FsPath,
    id: Uuid,
    offset: u64,
    user: Uuid,
    data: &[u8],
) -> Result<Manifest> {
    let mut m = owned(state, id, user)?;
    let length = data.len() as u64;
    let end = offset.checked_add(length).ok_or(QueueError::Invalid)?;
    if length == 0 || length > m.chunk_bytes {
        return Err(QueueError::Invalid.into());
    }
    let hash = checksum(data);
    if let Some(part) = m.parts.get(&offset) {
        if part.length != length || part.sha256 != hash {
            return Err(QueueError::Conflict.into());
        }
        if !part.received {
            enqueue(s, state, &m, offset, part)?;
        }
        return Ok(m);
    }
    if offset != m.uploaded
        || end > m.total_bytes
        || length != m.chunk_bytes.min(m.total_bytes.saturating_sub(offset))
    {
        return Err(QueueError::Conflict.into());
    }
    if m.parts
        .range(..offset)
        .next_back()
        .is_some_and(|(pos, p)| pos.saturating_add(p.length) > offset)
        || m.parts.range(offset..end).next().is_some()
    {
        return Err(QueueError::Conflict.into());
    }
    let folder = dir(state, id);
    let part = Part {
        length,
        sha256: hash,
        task_id: Uuid::new_v4(),
        received: false,
    };
    save_bytes(&folder.join(format!("input-{offset}")), data)?;
    m.parts.insert(offset, part.clone());
    m.uploaded = m.uploaded.checked_add(length).ok_or(QueueError::Invalid)?;
    files::save(&folder.join("manifest.json"), &m)?;
    enqueue(s, state, &m, offset, &part)?;
    Ok(m)
}
fn authorize_work(s: &Jobs, m: &Manifest, r: &ReadLease, node: &str) -> Result<Part> {
    s.lease(r.channel_id, &r.ids, node)?;
    let record = s.disk.record(r.ids.task_id)?;
    let expected: Work =
        serde_json::from_str(&record.task.input).map_err(|_| QueueError::Conflict)?;
    let part = m.parts.get(&r.offset).ok_or(QueueError::Missing)?;
    if record.status != Status::Leased
        || record.task.operation != Operation::Transfer
        || record.owner_id != Some(m.owner_id)
        || expected.session_id != m.session_id
        || expected.offset != r.offset
        || part.task_id != r.ids.task_id
    {
        return Err(QueueError::Conflict.into());
    }
    Ok(part.clone())
}
fn part_bytes(path: PathBuf, part: &Part) -> Result<Vec<u8>> {
    if fs::metadata(&path)?.len() != part.length || part.length > CHUNK as u64 {
        return Err("transfer length mismatch".into());
    }
    let data = fs::read(path)?;
    if checksum(&data) != part.sha256 {
        return Err("transfer checksum mismatch".into());
    }
    Ok(data)
}
pub fn routes() -> Router<Shared> {
    Router::new()
        .route("/v1/infinity/transfers", post(begin))
        .route("/v1/infinity/transfers/{id}", get(info))
        .route("/v1/infinity/transfers/{id}/parts/{offset}", put(part))
        .route("/v1/infinity/transfers/{id}/read", post(work))
        .route(
            "/v1/infinity/transfers/{id}/received/{offset}",
            get(received).put(deliver),
        )
        .layer(DefaultBodyLimit::max(CHUNK))
}
type Reply = std::result::Result<Json<Manifest>, Failure>;
type Binary = std::result::Result<([(&'static str, &'static str); 1], Vec<u8>), Failure>;
const OCTETS: [(&str, &str); 1] = [("content-type", "application/octet-stream")];
async fn begin(State(app): State<Shared>, h: HeaderMap, Json(body): Json<Create>) -> Reply {
    let user = crate::streams::user(app.clone(), &h).await?;
    Ok(Json(
        blocking(move || app.jobs.with(|_, p, _| create(p, body, user.id))).await?,
    ))
}
async fn info(State(app): State<Shared>, Path(id): Path<Uuid>, h: HeaderMap) -> Reply {
    let user = crate::streams::user(app.clone(), &h).await?;
    Ok(Json(
        blocking(move || app.jobs.with(|_, p, _| owned(p, id, user.id))).await?,
    ))
}
async fn part(
    State(app): State<Shared>,
    Path((id, offset)): Path<(Uuid, u64)>,
    h: HeaderMap,
    data: Bytes,
) -> Reply {
    let user = crate::streams::user(app.clone(), &h).await?;
    Ok(Json(
        blocking(move || {
            app.jobs
                .with(|s, p, _| upload(s, p, id, offset, user.id, &data))
        })
        .await?,
    ))
}
async fn work(
    State(app): State<Shared>,
    Path(id): Path<Uuid>,
    h: HeaderMap,
    Json(req): Json<ReadLease>,
) -> Binary {
    let node = super::node(&app, &h).await?;
    Ok((
        OCTETS,
        blocking(move || {
            app.jobs.with(|s, p, _| {
                let m = read(p, id)?;
                let part = authorize_work(s, &m, &req, &node)?;
                part_bytes(dir(p, id).join(format!("input-{}", req.offset)), &part)
            })
        })
        .await?,
    ))
}
async fn received(
    State(app): State<Shared>,
    Path((id, offset)): Path<(Uuid, u64)>,
    h: HeaderMap,
) -> Binary {
    let user = crate::streams::user(app.clone(), &h).await?;
    Ok((
        OCTETS,
        blocking(move || {
            app.jobs.with(|_, p, _| {
                let m = owned(p, id, user.id)?;
                let part = m
                    .parts
                    .get(&offset)
                    .filter(|p| p.received)
                    .ok_or(QueueError::Missing)?;
                part_bytes(dir(p, id).join(format!("received-{offset}")), part)
            })
        })
        .await?,
    ))
}
async fn deliver(
    State(app): State<Shared>,
    Path((id, offset)): Path<(Uuid, u64)>,
    h: HeaderMap,
    body: Bytes,
) -> Reply {
    let node = super::node(&app, &h).await?;
    let invalid = || Failure(StatusCode::BAD_REQUEST, "invalid transfer lease".into());
    let uuid = |name| {
        header(&h, name)
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or_else(invalid)
    };
    let req = ReadLease {
        channel_id: uuid("x-channel-id")?,
        offset,
        ids: LeaseRequest {
            task_id: uuid("x-task-id")?,
            worker_id: header(&h, "x-worker-id").ok_or_else(invalid)?.into(),
            lease_id: uuid("x-lease-id")?,
        },
    };
    Ok(Json(
        blocking(move || {
            app.jobs.with(|s, p, _| {
                let mut m = read(p, id)?;
                let part = authorize_work(s, &m, &req, &node)?;
                if part.length != body.len() as u64 || part.sha256 != checksum(&body) {
                    return Err(QueueError::Conflict.into());
                }
                if !part.received {
                    let folder = dir(p, id);
                    save_bytes(&folder.join(format!("received-{offset}")), &body)?;
                    m.parts.get_mut(&offset).unwrap().received = true;
                    m.received += part.length;
                    files::save(&folder.join("manifest.json"), &m)?;
                }
                Ok(m)
            })
        })
        .await?,
    ))
}
