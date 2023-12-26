use crate::{
    prelude::*,
    utils::{html::nav, PathBufExt, PathExt},
    Metadata,
};
use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse, Response},
};
use chrono::{DateTime, NaiveDate};
use convert_case::{Case, Casing};
use pulldown_cmark::Parser;
use pulldown_cmark_frontmatter::FrontmatterExtractor;
use std::{
    fmt::Write,
    fs::{self, read_dir},
    path::{Path, PathBuf},
};
use tokio::task::spawn_blocking;

#[derive(Template)]
#[template(path = "page.html")]
struct PageTemplate {
    title: String,
    last_modified: NaiveDate,
    content: String,
    nav: String,
}

#[derive(Template)]
#[template(path = "pic_grid.html")]
struct PicGridTemplate {
    img: String,
    name: String,
    link: String,
    caption: String,
}

pub fn render_markdown(State(state): State<AppState>, rel_path: PathBuf) -> Result<Response> {
    let fs_path = state.root.join(rel_path);
    let md = fs::read_to_string(&fs_path)?;
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
        nav: state.nav(),
    };
    Ok(Html(page.render()?).into_response())
}

pub async fn render_dir(State(state): State<AppState>, req_path: PathBuf) -> Result<Response> {
    let mut output = Vec::<String>::new();
    let req_path_fs = state.root.join(&req_path).canonicalize()?;
    // Filter out only valid files
    for entry in read_dir(state.root.join(&req_path))?
        .filter_map(|e| e.ok())
        .filter(filter_files)
    {
        let fname: PathBuf = entry
            .path()
            .file_name()
            .ok_or(Error::Generic(format!("Invalid path {:?}", entry)))?
            .into();
        let display_name = entry
            .path()
            .file_root()
            .ok_or(Error::Generic(format!("Invalid path {:?}", entry)))?
            .to_string()
            .to_case(Case::Title);

        // Get the different paths we need
        let path_fs = entry.path().canonicalize()?;
        let path = req_path.join(&fname);

        {
            use log::trace;
            trace!(" ---   Tracing paths used   ---");
            trace!("Request path:\t{}", req_path.display());
            trace!("Request path (fs):\t{}", req_path_fs.display());
            trace!("Entry path:\t\t{}", path.display());
            trace!("Entry path (fs):\t{}", path_fs.display());
            trace!("Display name:\t{}", display_name);
            trace!(
                "Dscr path (fs):\t{}",
                req_path_fs
                    .join(Path::new(&format!(
                        ".{}",
                        fname.with_extension("").display()
                    )))
                    .display()
            );
        }

        let img = path.with_extension("webp");
        output.push(match path_fs.with_extension("webp").is_file() {
            true => {
                let desc_path = req_path_fs.join(Path::new(&format!(
                    ".{}",
                    path.with_extension("").display()
                )));
                let caption = fs::read_to_string(desc_path).unwrap_or(String::new());
                let pg = PicGridTemplate {
                    img: format! {"/{}", img.display()},
                    name: display_name,
                    link: format!("/{}", path.display()),
                    caption,
                };
                pg.render()?
            }
            false => format!(r#"<p><a href="/{}">{display_name}</a></p>"#, path.display()),
        });
    }
    let title = req_path
        .file_root()
        .unwrap_or("Daniel's Website")
        .to_string()
        .to_case(Case::Title);
    let m = state.root.join(req_path).metadata()?;
    let last_modified: NaiveDate =
        <std::time::SystemTime as Into<DateTime<chrono::Utc>>>::into(m.modified()?).date_naive();
    let page = PageTemplate {
        title,
        last_modified,
        content: format!(r#"<div class="pic-grid">{}</div>"#, output.join("\n")),
        nav: state.nav(),
    };
    Ok(Html(page.render()?).into_response())
}

/// Return `true` if file is to be shown, `false` otherwise
fn filter_files(entry: &std::fs::DirEntry) -> bool {
    let is_md = entry
        .path()
        .extension()
        .map(|ext| "md" == ext)
        .unwrap_or(false);
    let is_dir = entry.file_type().map(|e| e.is_dir()).unwrap_or(false);
    let is_hidden = entry.path().is_hidden().unwrap_or(true); // Unwrap as true because on error,
                                                              // don't show file

    !is_hidden && (is_md || is_dir)
}
