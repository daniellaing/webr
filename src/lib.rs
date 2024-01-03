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
    extract::{Path, State},
    http::{Request, StatusCode, Uri},
    response::{Html, IntoResponse, Response},
    routing::get,
    BoxError, Router, ServiceExt,
};
use serde::Deserialize;
use std::path::PathBuf;
use templates::PageTemplate;
use time::OffsetDateTime;
use tokio::{fs::File, net::TcpListener, task::spawn_blocking};
use tokio_util::io::ReaderStream;
use tower::{util::MapRequestLayer, Layer};
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

async fn get_page(state: State<AppState>, Path(req_path): Path<PathBuf>) -> Response {
    let root = state.root.clone();
    get_page_wrapped(state, req_path)
        .await
        .unwrap_or_else(|err| {
            PageTemplate::builder()
                .title("Daniel's Website")
                .last_modified(OffsetDateTime::now_utc().date())
                .build(root, format!("{}<p>Error: {}</p>", ERROR_PAGE, err))
                .and_then(|ep| Ok(ep.render()?))
                .map(|ep| Html(ep).into_response())
                .unwrap_or(
                    (StatusCode::INTERNAL_SERVER_ERROR, Html(FALLBACK_ERROR)).into_response(),
                )
        })
}

async fn get_page_wrapped(state: State<AppState>, req_path: PathBuf) -> Result<Response> {
    let fs_path = state.root.join(&req_path);
    let ext = req_path.extension().and_then(std::ffi::OsStr::to_str);

    if fs_path.is_dir() {
        spawn_blocking(|| markdown::render_dir(state, req_path))
            .await?
            .map_err(Error::Markdown)
    } else if ext.is_none() {
        spawn_blocking(move || markdown::render_markdown(state, req_path.with_extension("md")))
            .await?
            .map_err(Error::Markdown)
    } else {
        get_file(state, req_path).await
    }
}

async fn get_file(State(state): State<AppState>, rel_path: PathBuf) -> Result<Response> {
    trace!(r#"Serving "{}""#, rel_path.display());
    let file = File::open(state.root.join(rel_path)).await?;
    let body = Body::from_stream(ReaderStream::new(file));
    let r = Response::builder().body(body)?;
    Ok(r)
}

static ERROR_PAGE: &str = r#"<h1>Oops!</h1><p>Something's not right with this page</p><p>It could be a problem with the server, or the page may simply not exist.</p><p>Try navigating back to the home page by clicking the "Home" button in the navigation bar.</p>"#;
static FALLBACK_ERROR: &str = r#"<!doctype html><html lang=en><meta charset=UTF-8><meta content="width=device-width,initial-scale=1" name=viewport><style>*,::after,::before{box-sizing:border-box;scroll-margin:5em 0 0;border-radius:.25em}:root{--max-width:80rem;--main-width:min(var(--max-width), 95vw);--fw-norm:300;--fw-bold:900;--ff-sans:"AlegreyaSans",sans-serif;--ff-mono:"Source Code Pro",monospace;--base-00:#292828;--base-01:#32302f;--base-02:#504945;--base-03:#665c54;--base-04:#bdae93;--base-06:#ddc7a1;--base-06:#ebdbb2;--base-07:#fbf1c7;--base-08:#ea6962;--base-09:#e78a4e;--base-0A:#d8a657;--base-0B:#a9b665;--base-0C:#89b482;--base-0D:#7daea3;--base-0E:#d3869b;--base-0F:#bd6f3e;--bs:0.25rem 0.25rem 0.75rem rgba(0, 0, 0, 0.25),0.125rem 0.125rem 0.25rem rgba(0, 0, 0, 0.15)}@font-face{font-family:AlegreyaSans;src:url(/.fonts/AlegreyaSans-Medium.eot);src:url(/.fonts/AlegreyaSans-Medium.woff) format("woff"),url(/.fonts/AlegreyaSans-Medium.woff2) format("woff2")}@supports (font-size:clamp(1rem,1vw,1rem)){:root{--fs--2:clamp(0.51rem, 0.23vw + 0.46rem, 0.74rem);--fs--1:clamp(0.61rem, 0.37vw + 0.54rem, 0.98rem);--fs-0:clamp(0.73rem, 0.58vw + 0.62rem, 1.31rem);--fs-1:clamp(0.88rem, 0.86vw + 0.71rem, 1.75rem);--fs-2:clamp(1.05rem, 1.27vw + 0.81rem, 2.33rem);--fs-3:clamp(1.26rem, 1.83vw + 0.92rem, 3.11rem);--fs-4:clamp(1.51rem, 2.6vw + 1.02rem, 4.15rem);--fs-5:clamp(1.81rem, 3.67vw + 1.13rem, 5.53rem)}}@supports not (font-size:clamp(1rem,1vw,1rem)){:root{--fs--2:0.51rem;--fs--1:0.61rem;--fs-0:0.73rem;--fs-1:0.88rem;--fs-2:1.05rem;--fs-3:1.26rem;--fs-4:1.51rem;--fs-5:1.81rem}@media screen and (min-width:1920px){:root{--fs--2:0.74rem;--fs--1:0.98rem;--fs-0:1.31rem;--fs-1:1.75rem;--fs-2:2.33rem;--fs-3:3.11rem;--fs-4:4.15rem;--fs-5:5.53rem}}}html{scroll-behaviour:smooth;margin:0;padding:0}body{background:var(--base-01);color:var(--base-06);font-family:var(--ff-sans);font-size:var(--fs-0);line-height:1.6;padding:0;margin:0;min-height:100vh;display:flex;flex-direction:column}main{width:var(--main-width);margin:5em auto 3em;padding:0 3em;position:relative;text-align:center}p{margin:1em 0 .5em 0}a{color:var(--base-06);opacity:1;position:relative;transition:opacity 75ms ease-in-out}a:hover{opacity:.7}h1{line-height:1;margin:1em 0 .5em 0;text-decoration:underline;margin-top:0;font-size:var(--fs-4);text-decoration-color:var(--base-08)}footer{background:var(--base-00);color:var(--base-06);text-align:center;font-size:var(--fs-1);padding:1em 0;margin:auto 0 0}footer a{color:inherit;font-size:var(--fw-bold)}footer ul{list-style:none;display:flex;justify-content:center;margin:2em 0 0;padding:0}footer ul li{margin:0 .5em}footer ul li a{padding:.5em}</style><link href=/style.css rel=stylesheet><title>Daniel's Website</title><main><h1>Fatal Error</h1><p>Something went wrong while trying to show the error page!<h2><a href=/ >Main page</a></h2></main><footer><a href=mailto:contact@daniellaing.com>contact@daniellaing.com</a><ul><li><a href=https://github.com/Bodleum , target=_blank>GitHub</a><li><a href=https://gitlab.com/Bodleum , target=_blank>GitLab</a><li><a href=https://www.instagram.com/_thebakerdan , target=_blank>Instagram</a></ul></footer>"#;
