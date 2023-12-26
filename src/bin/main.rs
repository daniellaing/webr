use webr::{prelude::*, start};

#[tokio::main]
async fn main() -> Result<()> {
    let state = AppState::builder()
        .root("./content")
        .md_options(Options::all())
        .build()
        .await?;

    start(state).await?;
    Ok(())
}
