#![allow(unused)]

pub mod error;
pub mod prelude;
mod utils;

use crate::prelude::*;
use axum::{extract::State, response::IntoResponse, routing::get, Router};
use std::{collections::HashMap, path::PathBuf};
use tokio::net::TcpListener;

#[derive(Debug, Clone)]
pub struct AppState {
    pub root: PathBuf,
}

pub async fn init_app(state: AppState) -> Result<(TcpListener, Router)> {
    env_logger::init();
    let app = init_router(state).await;
    let listener = TcpListener::bind("0.0.0.0:3000").await?;

    log::info!("Listening on http://{}", listener.local_addr()?);
    Ok((listener, app))
}

async fn init_router(state: AppState) -> Router {
    Router::new().route("/", get(get_page)).with_state(state)
}

async fn get_page(State(state): State<AppState>) -> String {
    format!("Content root: {}", state.root.display())
}
