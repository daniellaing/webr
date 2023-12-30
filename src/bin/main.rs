use webr::{prelude::*, start};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let state = AppState::builder()
        .root("./content")
        .md_options(Options::all())
        .build()
        .await?;

    start(state).await?;
    Ok(())
}
