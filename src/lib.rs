#![allow(unused)]

pub mod error;
mod markdown;
pub mod prelude;
mod utils;

use crate::prelude::*;
use askama::Template;
use axum::{
    extract::{Path, State},
    http::{Request, Uri},
    response::{Html, IntoResponse, Response},
    routing::get,
    serve::IncomingStream,
    Router, ServiceExt,
};
use pulldown_cmark::{html, Parser};
use pulldown_cmark_frontmatter::FrontmatterExtractor;
use serde::Deserialize;
use std::{collections::HashMap, convert::Infallible, fs, path::PathBuf};
use tokio::net::TcpListener;
use toml::value::Datetime;
use tower::{util::MapRequestLayer, Layer, Service};

#[derive(Debug, Clone)]
pub struct AppState {
    pub root: PathBuf,
    pub md_options: Options,
}

#[derive(Debug, Deserialize)]
struct Metadata {
    title: String,
    created: Datetime,
    modified: Datetime,
    tags: Vec<String>,
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
    if !uri.path().ends_with('/') && !uri.path().starts_with("//") {
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

async fn get_root(state: State<AppState>) -> Result<Html<String>> {
    get_page(state, Path(PathBuf::new())).await
}

async fn get_page(state: State<AppState>, Path(path): Path<PathBuf>) -> Result<Html<String>> {
    let fs_path = state.root.join(&path);
    if fs_path.is_dir() {
        markdown::render_dir(state, path)
    } else {
        markdown::render_markdown(state, fs::read_to_string(fs_path)?)
    }
}
