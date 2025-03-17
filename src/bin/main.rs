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

    let mut md_opts = Options::all();
    md_opts.remove(Options::ENABLE_SMART_PUNCTUATION);

    let state = AppState::builder()
        .root(args.content)
        .port(args.port)
        .md_options(md_opts)
        .build();

    start(state).await?;
    Ok(())
}
