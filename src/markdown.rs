use crate::{
    prelude::*,
    templates::{self, PageTemplate},
    utils::{is_shown, iterator::PartitionResult, path::PathExt},
    Metadata,
};
use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse, Response},
};
use convert_case::{Case, Casing};
use pulldown_cmark::Parser;
use pulldown_cmark_frontmatter::FrontmatterExtractor;
use std::{
    fs::{self, read_dir},
    path::Path,
    path::PathBuf,
};
use thiserror::Error;
use time::OffsetDateTime;

pub type Result<T> = core::result::Result<T, Error>;
#[derive(Debug, Error)]
pub enum Error {
    #[error("Could not get file root of {}", .0.display())]
    FileRoot(PathBuf),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error("No frontmatter found")]
    Frontmatter,

    #[error(transparent)]
    Toml(#[from] toml::de::Error),

    #[error(transparent)]
    Template(#[from] templates::Error),

    #[error(transparent)]
    Path(#[from] std::path::StripPrefixError),
}

#[derive(Template)]
#[template(path = "pic_grid.html")]
struct PicGridTemplate {
    img: String,
    name: String,
    link: String,
    caption: String,
}

#[derive(Debug, Default)]
struct Paths {
    entry_path: PathBuf,
    image_path: PathBuf,
    description_path: PathBuf,
    display_name: String,
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
            .ok_or(Error::Frontmatter)?
            .code_block
            .ok_or(Error::Frontmatter)?
            .source,
    )?;

    let l: OffsetDateTime = fs_path.metadata()?.modified()?.into();
    Ok(Html(
        PageTemplate::builder()
            .title(metadata.title)
            .last_modified(l.date())
            .build(state.root, content)?
            .render()
            .map_err(templates::Error::Template)?,
    )
    .into_response())
}

pub async fn render_dir(State(state): State<AppState>, req_path: PathBuf) -> Result<Response> {
    let req_path_fs = state.root.join(&req_path).canonicalize()?;
    // Filter out only valid files
    let (imgs, links) = read_dir(state.root.join(&req_path))?
        .filter_map(core::result::Result::ok)
        .filter(|e| is_shown(e).unwrap_or(false))
        .map(get_paths(&state.root, &req_path))
        .filter_map(core::result::Result::ok)
        .map(format_image_link(&state.root))
        // Separate any items which failed, just show link instead
        .partition_result();

    // Format links
    let links = links
        .into_iter()
        .map(format_links)
        .fold(String::new(), |acc, s| acc + &s);

    // Get page metadata
    let title = req_path
        .file_root()
        .unwrap_or("Daniel's Website")
        .to_string()
        .to_case(Case::Title);

    let l: OffsetDateTime = req_path_fs.metadata()?.modified()?.into();
    Ok(Html(
        PageTemplate::builder()
            .title(title)
            .last_modified(l.date())
            .build(
                state.root,
                format!(
                    r#"<div class="pic-grid">{}</div><ul>{}</ul>"#,
                    imgs.join(""),
                    links
                ),
            )?
            .render()
            .map_err(templates::Error::Template)?,
    )
    .into_response())
}

fn get_paths<'a>(
    root: &'a PathBuf,
    request_path: &'a PathBuf,
) -> impl FnMut(fs::DirEntry) -> Result<Paths> + 'a {
    move |e| {
        let entry_path = e.path().strip_prefix(root)?.to_path_buf();
        let display_name = entry_path
            .file_root()
            .ok_or(Error::FileRoot(e.path()))?
            .to_string()
            .to_case(Case::Title);
        let description_path = root.join(request_path).join(format!(
            ".{}",
            entry_path.file_root().ok_or(Error::FileRoot(e.path()))?
        ));

        Ok(Paths {
            image_path: entry_path.with_extension("webp"),
            entry_path,
            description_path,
            display_name,
        })
    }
}

fn format_image_link(root: &Path) -> impl FnMut(Paths) -> core::result::Result<String, Paths> + '_ {
    move |paths| {
        if !root.join(&paths.image_path).is_file() {
            return Err(paths);
        }

        let caption = fs::read_to_string(&paths.description_path).unwrap_or_default();
        let pg = PicGridTemplate {
            name: paths.display_name.clone(),
            img: format!("/{}", paths.image_path.display()),
            link: format!("/{}", paths.entry_path.display()),
            caption,
        };

        pg.render().map_err(|_e| paths)
    }
}

fn format_links(paths: Paths) -> String {
    format!(
        r#"<li><a href="/{}">{}</a></li>"#,
        paths.entry_path.display(),
        paths.display_name
    )
}
