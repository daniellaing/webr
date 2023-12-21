use std::path::PathBuf;

use webr::{init_app, prelude::*};

#[tokio::main]
async fn main() -> Result<()> {
    let state = AppState {
        root: PathBuf::from("./content"),
        md_options: Options::all(),
    };

    let (listener, app) = init_app(state).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
