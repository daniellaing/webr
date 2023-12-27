pub mod path;

use convert_case::{Case, Casing};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs::{read_dir, DirEntry};
use tokio_stream::{wrappers::ReadDirStream, StreamExt};

use crate::{prelude::*, utils::path::PathExt};

use self::path::PathBufExt;

pub async fn nav(root: impl AsRef<Path>) -> Result<String> {
    let mut output = Vec::new();
    let mut stream = ReadDirStream::new(read_dir(root).await?).filter_map(|e| e.ok());
    while let Some(entry) = stream.next().await {
        if is_shown_dir_only(&entry).await.unwrap_or(false) {
            let entry_path = entry.path();
            let display_name = entry_path
                .file_root()
                .ok_or(Error::FileRoot(format!("{}", entry_path.display())))?
                .to_string()
                .to_case(Case::Title);
            let path: PathBuf = entry_path
                .file_name()
                .ok_or(Error::FileName(format!("{}", entry_path.display())))?
                .into();
            output.push(format!(
                r#"<li><a href="/{}">{display_name}</a></li>"#,
                path.display()
            ));
        };
    }

    Ok(format!(
        r#"<li><a href="/">Home</a></li>{}"#,
        output.join("\n")
    ))
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Could not get file root of {0}")]
    FileRoot(String),

    #[error("Could not get file name of {0}")]
    FileName(String),
}

pub async fn is_shown(entry: &DirEntry) -> Result<bool> {
    let hidden = entry.path().is_hidden()?;
    let is_dir = entry.file_type().await.map(|e| e.is_dir())?;
    let is_md = entry
        .path()
        .extension()
        .map(|ext| "md" == ext)
        .unwrap_or(false);

    Ok(!hidden && (is_md || is_dir))
}

pub async fn is_shown_dir_only(entry: &DirEntry) -> Result<bool> {
    let hidden = entry.path().is_hidden()?;
    let is_dir = entry.file_type().await.map(|e| e.is_dir())?;

    Ok(!hidden && is_dir)
}
