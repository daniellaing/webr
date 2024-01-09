use crate::prelude::*;
use std::path::PathBuf;
use tracing::trace;

#[derive(Debug, Clone)]
pub struct AppState {
    pub root: PathBuf,
    pub md_options: Options,
    pub port: u16,
}

impl AppState {
    pub fn builder() -> AppStateBuilder<NoRoot, NoPort> {
        trace!("Building AppState");
        AppStateBuilder::default()
    }
}

#[derive(Default)]
pub struct AppStateBuilder<R, P> {
    root: R,
    md_options: Option<Options>,
    port: P,
}

impl AppStateBuilder<NoRoot, NoPort> {
    pub fn new() -> Self {
        AppStateBuilder::default()
    }
}

impl<P> AppStateBuilder<NoRoot, P> {
    pub fn root(self, root: impl Into<PathBuf>) -> AppStateBuilder<Root, P> {
        trace!("Setting content root");
        AppStateBuilder {
            root: Root(root.into()),
            md_options: self.md_options,
            port: self.port,
        }
    }
}

impl AppStateBuilder<Root, Port> {
    pub fn build(self) -> AppState {
        trace!("Finished building AppState");
        AppState {
            root: self.root.0,
            md_options: self.md_options.unwrap_or(Options::empty()),
            port: self.port.0,
        }
    }
}

impl<R> AppStateBuilder<R, NoPort> {
    pub fn port(self, port: u16) -> AppStateBuilder<R, Port> {
        trace!("Setting port");
        AppStateBuilder {
            root: self.root,
            md_options: self.md_options,
            port: Port(port),
        }
    }
}

impl<R, P> AppStateBuilder<R, P> {
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
#[derive(Default, Clone)]
pub struct NoPort;
#[derive(Default, Clone)]
pub struct Port(u16);
