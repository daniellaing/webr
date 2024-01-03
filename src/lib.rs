#![allow(unused)]

pub mod app_state;
pub mod error;
mod markdown;
pub mod prelude;
mod templates;
mod utils;

use crate::prelude::*;
use askama::Template;
use axum::{
    body::Body,
    error_handling::HandleErrorLayer,
    extract::{Path, State},
    http::{Request, StatusCode, Uri},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router, ServiceExt,
};
use serde::Deserialize;
use std::path::PathBuf;
use templates::{PageTemplate, PageTemplateBuilder};
use time::{Date, Duration, OffsetDateTime};
use tokio::{fs::File, net::TcpListener, task::spawn_blocking};
use tokio_util::io::ReaderStream;
use tower::{util::MapRequestLayer, BoxError, Layer, ServiceBuilder};
use tower_http::trace::TraceLayer;
use tracing::{debug, trace};

#[derive(Debug, Deserialize)]
struct Metadata {
    title: String,
    tags: Option<Vec<String>>,
}

impl Default for Metadata {
    fn default() -> Self {
        Metadata {
            title: String::from("Daniel's Website"),
            tags: None,
        }
    }
}

pub async fn start(state: AppState) -> Result<()> {
    debug!("Creating Router");
    let app = MapRequestLayer::new(normalize_path).layer(
        Router::new()
            .route("/", get(get_root))
            .route("/*path", get(get_page))
            .layer(TraceLayer::new_for_http())
            .with_state(state),
    );
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("Listening on http://{}", listener.local_addr()?);
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

    tracing::trace!("Triming trailing slash from {}", uri);
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

async fn get_root(state: State<AppState>) -> Response {
    get_page(state, Path(PathBuf::new())).await
}

async fn get_page(state: State<AppState>, Path(rel_path): Path<PathBuf>) -> Response {
    let fs_path = state.root.join(&rel_path);
    let root = state.root.clone(); // For use in error
    let ext = rel_path.extension().and_then(std::ffi::OsStr::to_str);

    if fs_path.is_dir() {
        spawn_blocking(|| markdown::render_dir(state, rel_path))
            .await
            .map_err(|err| Error::TokioJoinError(err))
            .and_then(|a| -> Result<Response> { Ok(a?) })
    } else if ext.is_none() {
        spawn_blocking(move || markdown::render_markdown(state, rel_path.with_extension("md")))
            .await
            .map_err(|err| Error::TokioJoinError(err))
            .and_then(|a| -> Result<Response> { Ok(a?) })
    } else {
        get_file(state, rel_path).await
    }
    .unwrap_or_else(|err| handle_error(root, err))
}

async fn get_file(State(state): State<AppState>, rel_path: PathBuf) -> Result<Response> {
    trace!(r#"Serving "{}""#, rel_path.display());
    let file = File::open(state.root.join(rel_path)).await?;
    let body = Body::from_stream(ReaderStream::new(file));
    let r = Response::builder().body(body)?;
    Ok(r)
}

fn handle_error(root: impl AsRef<std::path::Path>, err: Error) -> Response {
    static CONTENT: &str = r#"<h1 style="text-align:center;">This page doesn't exist!</h1>"#;
    Html(
        PageTemplate::builder()
            .title("Daniel's Website")
            .last_modified(OffsetDateTime::now_utc().date())
            .build(root, CONTENT)
            .and_then(|p| Ok(p.render()?))
            .expect("Failed to render error page"),
    )
    .into_response()
}
