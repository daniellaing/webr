#![allow(unused)]

pub mod error;
pub mod prelude;
mod utils;

use crate::prelude::*;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use pulldown_cmark::{html, Parser};
use std::{collections::HashMap, fs, path::PathBuf};
use tokio::net::TcpListener;

#[derive(Debug, Clone)]
pub struct AppState {
    pub root: PathBuf,
    pub md_options: Options,
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

async fn get_root(state: State<AppState>) -> impl IntoResponse {
    get_page(state, Path(PathBuf::new())).await
}

async fn get_page(State(state): State<AppState>, Path(path): Path<PathBuf>) -> Result<String> {
    let path = state.root.join(&path);
    let md = fs::read_to_string(&path)?;
    let parser = Parser::new_ext(&md, state.md_options);
    let mut content = String::new();
    html::push_html(&mut content, parser);

    Ok(format!(
        "Content root: {}\nPath: {}\nContent:\n\n{}",
        state.root.display(),
        path.display(),
        content
    ))
}
