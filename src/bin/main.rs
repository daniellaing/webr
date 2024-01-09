#![recursion_limit = "512"]

use clap::Parser;
use webr::{prelude::*, start};

#[tokio::main]
async fn main() -> R<()> {
    tracing_subscriber::fmt()
        .pretty()
        .with_max_level(tracing::Level::TRACE)
        .init();

    let args = Args::parse();

    let state = AppState::builder()
        .root(args.content)
        .port(args.port)
        .md_options(Options::all())
        .build();

    start(state).await?;
    Ok(())
}
