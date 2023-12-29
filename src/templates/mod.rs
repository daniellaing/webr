use crate::utils::{self, nav};
use askama::Template;
use std::path::Path;
use thiserror::Error;
use time::Date;

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
        PageTemplateBuilder {
            title: Title(title.into()),
            last_modified: self.last_modified,
        }
    }
}

impl<T> PageTemplateBuilder<T, NoLM> {
    pub fn last_modified(self, last_modified: impl Into<Date>) -> PageTemplateBuilder<T, LM> {
        PageTemplateBuilder {
            title: self.title,
            last_modified: LM(last_modified.into()),
        }
    }
}

impl PageTemplateBuilder<Title, LM> {
    pub fn build(self, root: impl AsRef<Path>, content: impl Into<String>) -> R<PageTemplate> {
        Ok(PageTemplate {
            title: self.title.0,
            content: content.into(),
            last_modified: self.last_modified.0,
            nav: nav(root)?,
        })
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
