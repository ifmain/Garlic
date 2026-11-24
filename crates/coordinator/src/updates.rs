use crate::http::{bearer, blocking, Failure, Shared};
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use tokio_util::io::ReaderStream;

fn published_name(name: &str) -> bool {
    matches!(
        name,
        "android-aarch64.json" | "android-arm.json" | "android-x86_64.json" | "windows-x86_64.json"
    ) || (name.len() == 64
        && name
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)))
}

pub async fn get(
    State(app): State<Shared>,
    Path(name): Path<String>,
    headers: HeaderMap,
) -> Result<Response, Failure> {
    let token = bearer(&headers)?.to_owned();
    let auth = app.clone();
    blocking(move || auth.accounts.user(&token).map(|_| ())).await?;
    let Some(dir) = app.update_dir.as_ref().filter(|_| published_name(&name)) else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };
    let path = dir.join(name);
    if !tokio::fs::symlink_metadata(&path)
        .await
        .is_ok_and(|m| m.is_file())
    {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }
    let mut options = tokio::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    let Ok(file) = options.open(path).await else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };
    let Ok(meta) = file.metadata().await else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };
    if !meta.is_file() {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }
    let mut response = Body::from_stream(ReaderStream::new(file)).into_response();
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, "private, no-store".parse().unwrap());
    response
        .headers_mut()
        .insert(header::CONTENT_LENGTH, meta.len().into());
    Ok(response)
}
