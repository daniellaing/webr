use webr::{init_app, prelude::*};

#[tokio::main]
async fn main() -> Result<()> {
    let (listener, app) = init_app().await?;
    axum::serve(listener, app).await?;
    Ok(())
}
