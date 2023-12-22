use std::path::PathBuf;

use webr::{prelude::*, start};

#[tokio::main]
async fn main() -> Result<()> {
    let state = AppState {
        root: PathBuf::from("./content"),
        md_options: Options::all(),
    };

    start(state).await?;
    Ok(())
}
