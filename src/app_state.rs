use std::path::{Path, PathBuf};

use crate::{prelude::*, utils::html::nav};

#[derive(Debug, Clone)]
pub struct AppState {
    pub root: PathBuf,
    pub md_options: Options,
    nav: String,
}

impl AppState {
    pub fn builder() -> AppStateBuilder<NoRoot> {
        AppStateBuilder::default()
    }

    pub fn nav(self) -> String {
        self.nav
    }
}

#[derive(Default)]
pub struct AppStateBuilder<R> {
    root: R,
    md_options: Option<Options>,
}

impl AppStateBuilder<NoRoot> {
    pub fn new() -> Self {
        AppStateBuilder::default()
    }

    pub fn root(mut self, root: impl Into<PathBuf>) -> AppStateBuilder<Root> {
        AppStateBuilder {
            root: Root(root.into()),
            md_options: self.md_options,
        }
    }
}

impl AppStateBuilder<Root> {
    pub async fn build(self) -> Result<AppState> {
        Ok(AppState {
            nav: nav(&self.root.0).await?,
            root: self.root.0,
            md_options: self.md_options.unwrap_or(Options::empty()),
        })
    }
}

impl<R> AppStateBuilder<R> {
    pub fn md_options(mut self, md_options: Options) -> Self {
        self.md_options = Some(md_options);
        self
    }
}

// TypeState
#[derive(Default, Clone)]
pub struct NoRoot;
#[derive(Default, Clone)]
pub struct Root(PathBuf);
