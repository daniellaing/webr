use crate::{
    build_error_page,
    prelude::*,
    templates::{self, PageTemplate},
};
use askama::Template;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse, Response},
};
use std::path::PathBuf;
use thiserror::Error;
use time::{Date, OffsetDateTime};
use tokio::task::spawn_blocking;

pub type R<T> = core::result::Result<T, Error>;
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Template(#[from] templates::Error),
}

pub async fn lectionary(state: State<AppState>) -> Response {
    let root = state.root.clone();
    lectionary_wrapped(state)
        .await
        .unwrap_or_else(|err| build_error_page(root, err.into()))
}

async fn lectionary_wrapped(state: State<AppState>) -> R<Response> {
    Ok(Html(
        PageTemplate::builder()
            .title("Daniel's Lectionary")
            .last_modified(OffsetDateTime::now_utc().date())
            .tags(vec![String::from("lectionary"), String::from("bible")])
            .build(&state.root, "Lectionary")?
            .render()
            .map_err(templates::Error::Template)?,
    )
    .into_response())
}
