use crate::{
    prelude::*,
    utils::{PathBufExt, PathExt},
};
use axum::extract::State;
use convert_case::{Case, Casing};
use std::path::{Path, PathBuf};
use tokio::fs::read_dir;

pub async fn nav(root: impl AsRef<Path>) -> Result<String> {
    let mut output = Vec::new();

    let mut reader = read_dir(root).await?;
    let mut next = reader.next_entry().await?;
    while let Some(entry) = next {
        if entry.file_type().await?.is_dir() && !entry.path().is_hidden()? {
            let display_name = entry
                .path()
                .file_root()
                .ok_or(Error::Generic(format!("Invalid path {:?}", entry)))?
                .to_string()
                .to_case(Case::Title);
            let path: PathBuf = entry
                .path()
                .file_name()
                .ok_or(Error::Generic(format!("Invalid path {:?}", entry)))?
                .into();
            dbg!(&display_name);
            output.push(format!(
                r#"<li><a href="/{}">{display_name}</a></li>"#,
                path.display()
            ));
        }
        next = reader.next_entry().await?;
    }

    Ok(format!(
        r#"<ul><li><a href="/">Home</a></li>{}</ul>"#,
        output.join("\n")
    ))
}
