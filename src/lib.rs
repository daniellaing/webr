#![allow(unused)]

pub mod app_state;
pub mod error;
mod markdown;
pub mod prelude;
mod utils;

use crate::prelude::*;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{Request, Uri},
    response::{IntoResponse, Response},
    routing::get,
    Router, ServiceExt,
};
use serde::Deserialize;
use std::path::PathBuf;
use tokio::{fs::File, net::TcpListener, task::spawn_blocking};
use tokio_util::io::ReaderStream;
use tower::{util::MapRequestLayer, Layer};

#[derive(Debug, Deserialize)]
struct Metadata {
    title: String,
    tags: Option<Vec<String>>,
}

pub async fn init(root: PathBuf) -> Result<AppState> {
    todo!()
}

pub async fn start(state: AppState) -> Result<()> {
    env_logger::init();

    let app = MapRequestLayer::new(normalize_path).layer(
        Router::new()
            .route("/", get(get_root))
            .route("/*path", get(get_page))
            .with_state(state),
    );
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    log::info!("Listening on http://{}", listener.local_addr()?);
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

fn normalize_path<B>(mut req: Request<B>) -> Request<B>
where
    B: std::fmt::Debug,
{
    let uri = req.uri_mut();
    // If no trailing slash, just proceed
    if !uri.path().ends_with('/') && !uri.path().starts_with("//") || uri.path() == "/" {
        return req;
    }

    log::trace!("Triming trailing slash from {}", uri);
    // Trim the trailing slash
    let new_path = format!("/{}", uri.path().trim_matches('/'));

    // Write new uri
    let mut parts = uri.clone().into_parts();
    let new_pq = if let Some(pq) = parts.path_and_query {
        let q = if let Some(q) = pq.query() {
            format!("?{q}")
        } else {
            String::new()
        };
        Some(
            format!("{new_path}{q}")
                .parse()
                .expect("Error parsing rewritten uri"),
        )
    } else {
        None
    };

    // Rewrite request
    parts.path_and_query = new_pq;
    if let Ok(new_uri) = Uri::from_parts(parts) {
        *uri = new_uri;
    }
    req
}

async fn get_root(state: State<AppState>) -> Result<impl IntoResponse> {
    get_page(state, Path(PathBuf::new())).await
}

async fn get_page(state: State<AppState>, Path(rel_path): Path<PathBuf>) -> Result<Response> {
    let fs_path = state.root.join(&rel_path);
    let ext = rel_path.extension().and_then(std::ffi::OsStr::to_str);

    if fs_path.is_dir() {
        markdown::render_dir(state, rel_path).await
    } else if ext == Some("md") {
        spawn_blocking(|| markdown::render_markdown(state, rel_path)).await?
    } else {
        get_file(state, rel_path).await
    }
}

async fn get_file(State(state): State<AppState>, rel_path: PathBuf) -> Result<Response> {
    let file = File::open(state.root.join(rel_path)).await?;
    let body = Body::from_stream(ReaderStream::new(file));
    let r = Response::builder().body(body)?;
    Ok(r)
}
