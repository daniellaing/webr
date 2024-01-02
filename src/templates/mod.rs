use crate::utils::{self, nav};
use askama::Template;
use std::path::Path;
use thiserror::Error;
use time::Date;
use tracing::{debug, trace};

pub type R<T> = Result<T, Error>;
#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to build nav")]
    Nav(#[from] utils::Error),

    #[error("Failed to render template")]
    Template(#[from] askama::Error),
}

#[derive(Template, Debug)]
#[template(path = "page.html")]
pub struct PageTemplate {
    title: String,
    last_modified: Date,
    content: String,
    nav: String,
}

impl PageTemplate {
    pub fn builder() -> PageTemplateBuilder<NoTitle, NoLM> {
        debug!(r#"Building page template"#);
        PageTemplateBuilder::default()
    }
}

#[derive(Debug, Default)]
pub struct PageTemplateBuilder<T, M> {
    title: T,
    last_modified: M,
}

impl<M> PageTemplateBuilder<NoTitle, M> {
    pub fn title(self, title: impl Into<String>) -> PageTemplateBuilder<Title, M> {
        let title = title.into();
        trace!(r#"Adding page title: "{}""#, &title);
        PageTemplateBuilder {
            title: Title(title),
            last_modified: self.last_modified,
        }
    }
}

impl<T> PageTemplateBuilder<T, NoLM> {
    pub fn last_modified(self, last_modified: impl Into<Date>) -> PageTemplateBuilder<T, LM> {
        trace!("Adding page last modified date");
        PageTemplateBuilder {
            title: self.title,
            last_modified: LM(last_modified.into()),
        }
    }
}

impl PageTemplateBuilder<Title, LM> {
    pub fn build(self, root: impl AsRef<Path>, content: impl Into<String>) -> R<PageTemplate> {
        let pt = PageTemplate {
            title: self.title.0,
            content: content.into(),
            last_modified: self.last_modified.0,
            nav: nav(root)?,
        };
        Ok(pt)
    }
}

// TypeState
#[derive(Default, Clone)]
pub struct NoTitle;
#[derive(Default, Clone)]
pub struct Title(String);
#[derive(Default, Clone)]
pub struct NoLM;
#[derive(Clone)]
pub struct LM(Date);
