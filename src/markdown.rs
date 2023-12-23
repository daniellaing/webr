use crate::{
    prelude::*,
    utils::{PathBufExt, PathExt},
    Metadata,
};
use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse, Response},
};
use pulldown_cmark::Parser;
use pulldown_cmark_frontmatter::FrontmatterExtractor;
use std::{fmt::Write, fs::read_dir, path::PathBuf};

#[derive(Template)]
#[template(path = "page.html")]
struct PageTemplate {
    title: String,
    content: String,
}

pub fn render_markdown(State(state): State<AppState>, md: String) -> Result<Response> {
    let mut extractor = FrontmatterExtractor::new(Parser::new_ext(&md, state.md_options));
    let mut content = String::new();
    pulldown_cmark::html::push_html(&mut content, &mut extractor);

    let metadata: Metadata = toml::from_str(
        &extractor
            .frontmatter
            .ok_or(Error::Generic(String::from("No frontmatter found")))?
            .code_block
            .ok_or(Error::Generic(String::from("No codeblock found")))?
            .source,
    )?;

    let page = PageTemplate {
        title: metadata.title,
        content,
    };
    Ok(Html(page.render()?).into_response())
}

pub fn render_dir(State(state): State<AppState>, path: PathBuf) -> Result<Response> {
    let mut output = Vec::<String>::new();
    // Filter out only valid files
    for entry in read_dir(state.root.join(&path))?
        .filter_map(|e| e.ok())
        .filter(filter_files)
    {
        let fname: PathBuf = entry
            .path()
            .file_name()
            .ok_or(Error::Generic(format!("Invalid path {:?}", entry)))?
            .into();
        let display = entry
            .path()
            .file_root()
            .ok_or(Error::Generic(format!("Invalid path {:?}", entry)))?
            .to_string();
        output.push(format!(
            r#"<li><a href="/{}">{display}</a></li>"#,
            path.join(&fname).display()
        ));
    }

    let title = path.file_root().unwrap_or("Daniel's Website").to_string();
    let page = PageTemplate {
        title,
        content: format!("<ul>{}</ul>", output.join("\n")),
    };
    Ok(Html(page.render()?).into_response())
}

/// Return `true` if file is to be shown, `false` otherwise
fn filter_files(entry: &std::fs::DirEntry) -> bool {
    // Never show hidden files
    if entry.path().is_hidden().unwrap_or(false) {
        false
    } else {
        // Is dir or md file
        entry.file_type().ok().map(|e| e.is_dir()).unwrap_or(false)
            || entry
                .path()
                .extension()
                .and_then(|ext| Some("md" == ext))
                .unwrap_or(false)
    }
}
