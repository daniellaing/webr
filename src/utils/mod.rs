pub mod iterator;
pub mod path;

use convert_case::{Case, Casing};
use std::{
    fs::{read_dir, DirEntry},
    path::{Path, PathBuf},
};
use thiserror::Error;
use tracing::trace;

use crate::utils::path::PathExt;

pub type Result<T> = core::result::Result<T, Error>;
#[derive(Debug, Error)]
pub enum Error {
    #[error("Could not get file root of {}", .0.display())]
    FileRoot(PathBuf),

    #[error("Could not get file name of {}", .0.display())]
    FileName(PathBuf),

    #[error("Invalid path: {}", .0.display())]
    InvalidPath(PathBuf),

    #[error("Nav is empty")]
    EmptyNav,

    #[error(transparent)]
    IO(#[from] std::io::Error),
}

pub fn nav(root: impl AsRef<Path>) -> Result<String> {
    trace!("Building nav");
    let nav_home_link = String::from(r#"<li><a href="/">Home</a></li>"#);
    let mut entries = read_dir(root)?
        .filter_map(core::result::Result::ok)
        .filter(|e| is_shown_dir_only(e).unwrap_or(false))
        .map(to_display_and_fname)
        .filter_map(core::result::Result::ok)
        .collect::<Vec<_>>();
    entries.sort_by(|a, b| natord::compare(&a.0, &b.0));
    Ok(entries
        .into_iter()
        .map(|(display_name, path)| {
            format!(
                r#"<li><a href="/{}">{display_name}</a></li>"#,
                path.display()
            )
        })
        .fold(nav_home_link, |acc, e| acc + &e))
}

pub fn is_shown(entry: &DirEntry) -> Result<bool> {
    let hidden = entry
        .path()
        .is_hidden()
        .ok_or(Error::InvalidPath(entry.path()))?;
    let is_dir = entry.file_type().map(|e| e.is_dir())?;
    let is_md = entry
        .path()
        .extension()
        .map(|ext| "md" == ext)
        .unwrap_or(false);

    Ok(!hidden && (is_md || is_dir))
}

pub fn is_shown_dir_only(entry: &DirEntry) -> Result<bool> {
    let hidden = entry
        .path()
        .is_hidden()
        .ok_or(Error::InvalidPath(entry.path()))?;
    let is_dir = entry.file_type().map(|e| e.is_dir())?;

    Ok(!hidden && is_dir)
}

fn to_display_and_fname(entry: DirEntry) -> Result<(String, PathBuf)> {
    let path: PathBuf = entry
        .path()
        .file_name()
        .ok_or(Error::FileName(entry.path()))?
        .into();
    let display_name = path
        .file_root()
        .map(String::from)
        .map(|s| s.to_case(Case::Title))
        .ok_or(Error::FileRoot(entry.path()))?;
    Ok((display_name, path))
}
