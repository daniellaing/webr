#![allow(unused)]

pub mod error;
pub mod prelude;
mod utils;

use std::collections::HashMap;

use axum::{response::IntoResponse, routing::get, Router};
use tokio::net::TcpListener;

use crate::prelude::*;

pub async fn init_app() -> Result<(TcpListener, Router)> {
    env_logger::init();
    let app = init_router().await;
    let listener = TcpListener::bind("0.0.0.0:3000").await?;

    log::info!("Listening on http://{}", listener.local_addr()?);
    Ok((listener, app))
}

async fn init_router() -> Router<()> {
    Router::new().route("/", get(get_page))
}

async fn get_page() -> String {
    String::from("Placeholder")
}
