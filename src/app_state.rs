use crate::{prelude::*, utils::nav};
use std::path::PathBuf;
use tracing::trace;

#[derive(Debug, Clone)]
pub struct AppState {
    pub root: PathBuf,
    pub md_options: Options,
}

impl AppState {
    pub fn builder() -> AppStateBuilder<NoRoot> {
        trace!("Building AppState");
        AppStateBuilder::default()
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

    pub fn root(self, root: impl Into<PathBuf>) -> AppStateBuilder<Root> {
        trace!("Setting content root");
        AppStateBuilder {
            root: Root(root.into()),
            md_options: self.md_options,
        }
    }
}

impl AppStateBuilder<Root> {
    pub fn build(self) -> AppState {
        trace!("Finished building AppState");
        AppState {
            root: self.root.0,
            md_options: self.md_options.unwrap_or(Options::empty()),
        }
    }
}

impl<R> AppStateBuilder<R> {
    pub fn md_options(mut self, md_options: Options) -> Self {
        trace!("Setting markdown parsing options");
        self.md_options = Some(md_options);
        self
    }
}

// TypeState
#[derive(Default, Clone)]
pub struct NoRoot;
#[derive(Default, Clone)]
pub struct Root(PathBuf);
