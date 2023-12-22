use crate::{prelude::*, Metadata};
use askama::Template;
use axum::{extract::State, response::Html};
use pulldown_cmark::Parser;
use pulldown_cmark_frontmatter::FrontmatterExtractor;
use std::{fmt::Write, fs::read_dir, path::PathBuf};

#[derive(Template)]
#[template(path = "page.html")]
struct PageTemplate {
    title: String,
    content: String,
}

pub fn render_markdown(State(state): State<AppState>, md: String) -> Result<Html<String>> {
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
    Ok(Html(page.render()?))
}

pub fn render_dir(State(state): State<AppState>, path: PathBuf) -> Result<Html<String>> {
    dbg!(&path);

    let mut output = Vec::<String>::new();
    for entry in read_dir(state.root.join(&path))?.filter_map(|e| e.ok()) {
        let fname: PathBuf = entry
            .path()
            .file_name()
            .ok_or(Error::Generic(format!("Invalid path {:?}", entry)))?
            .into();
        let display = fname
            .file_stem()
            .ok_or(Error::Generic(format!("Invalid path {:?}", entry)))?
            .to_str()
            .ok_or(Error::Generic(format!("Invalid path {:?}", entry)))?;
        output.push(format!(
            r#"<a href="/{}">{display}</a>"#,
            path.join(&fname).display()
        ));
    }

    Ok(Html(output.join("\n")))
}
