#![allow(unused)]

pub mod error;
mod markdown;
pub mod prelude;
mod utils;

use crate::prelude::*;
use askama::Template;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use pulldown_cmark::{html, Parser};
use pulldown_cmark_frontmatter::FrontmatterExtractor;
use serde::Deserialize;
use std::{collections::HashMap, fs, path::PathBuf};
use tokio::net::TcpListener;
use toml::value::Datetime;

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

pub async fn init_app(state: AppState) -> Result<(TcpListener, Router)> {
    env_logger::init();
    let app = init_router(state).await;
    let listener = TcpListener::bind("0.0.0.0:3000").await?;

    log::info!("Listening on http://{}", listener.local_addr()?);
    Ok((listener, app))
}

async fn init_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(get_root))
        .route("/*path", get(get_page))
        .with_state(state)
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
