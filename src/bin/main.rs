#![recursion_limit = "512"]

use webr::{prelude::*, start};

#[tokio::main]
async fn main() -> R<()> {
    tracing_subscriber::fmt()
        .pretty()
        .with_max_level(tracing::Level::TRACE)
        .init();

    let state = AppState::builder()
        .root("./content")
        .md_options(Options::all())
        .build();

    start(state).await?;
    Ok(())
}
