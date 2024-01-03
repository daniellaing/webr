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
    last_modified: String,
    content: String,
    nav: String,
}

impl PageTemplate {
    pub fn builder() -> PageTemplateBuilder<NoTitle> {
        debug!(r#"Building page template"#);
        PageTemplateBuilder::default()
    }
}

#[derive(Debug, Default)]
pub struct PageTemplateBuilder<T> {
    title: T,
    last_modified: Option<Date>,
}

impl PageTemplateBuilder<NoTitle> {
    pub fn title(self, title: impl Into<String>) -> PageTemplateBuilder<Title> {
        let title = title.into();
        trace!(r#"Adding page title: "{}""#, &title);
        PageTemplateBuilder {
            title: Title(title),
            last_modified: self.last_modified,
        }
    }
}

impl<T> PageTemplateBuilder<T> {
    pub fn last_modified(self, last_modified: impl Into<Option<Date>>) -> PageTemplateBuilder<T> {
        trace!("Adding page last modified date");
        PageTemplateBuilder {
            title: self.title,
            last_modified: last_modified.into(),
        }
    }
}

impl PageTemplateBuilder<Title> {
    pub fn build(self, root: impl AsRef<Path>, content: impl Into<String>) -> R<PageTemplate> {
        let last_modified = match self.last_modified {
            Some(d) => format!(r#"<p class="last_modified">Last updated: {}</p>"#, d),
            None => String::new(),
        };
        let pt = PageTemplate {
            title: self.title.0,
            content: content.into(),
            last_modified,
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
