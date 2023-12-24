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
use chrono::{DateTime, NaiveDate};
use pulldown_cmark::Parser;
use pulldown_cmark_frontmatter::FrontmatterExtractor;
use std::{fmt::Write, fs::read_dir, path::PathBuf};
use tokio::fs;

#[derive(Template)]
#[template(path = "page.html")]
struct PageTemplate {
    title: String,
    last_modified: NaiveDate,
    content: String,
}

pub async fn render_markdown(State(state): State<AppState>, rel_path: PathBuf) -> Result<Response> {
    let fs_path = state.root.join(&rel_path);
    let md = fs::read_to_string(&fs_path).await?;
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

    let m = fs_path.metadata()?;
    let last_modified: NaiveDate =
        <std::time::SystemTime as Into<DateTime<chrono::Utc>>>::into(m.modified()?).date_naive();

    let page = PageTemplate {
        title: metadata.title,
        last_modified,
        content,
    };
    Ok(Html(page.render()?).into_response())
}

pub fn render_dir(State(state): State<AppState>, rel_path: PathBuf) -> Result<Response> {
    let mut output = Vec::<String>::new();
    // Filter out only valid files
    for entry in read_dir(state.root.join(&rel_path))?
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

        let img = rel_path.join(&fname).with_extension("webp");
        output.push(match state.root.join(&img).is_file() {
            true => format!(
                r#"<li><a href="/{}"><img src="/{}" alt="{display}"></img></a></li>"#,
                rel_path.join(&fname).display(),
                img.display()
            ),
            false => format!(
                r#"<li><a href="/{}">{display}</a></li>"#,
                rel_path.join(&fname).display()
            ),
        });
    }

    let title = rel_path
        .file_root()
        .unwrap_or("Daniel's Website")
        .to_string();
    let m = state.root.join(rel_path).metadata()?;
    let last_modified: NaiveDate =
        <std::time::SystemTime as Into<DateTime<chrono::Utc>>>::into(m.modified()?).date_naive();
    let page = PageTemplate {
        title,
        last_modified,
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
