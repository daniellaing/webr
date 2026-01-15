use crate::{
    prelude::*,
    templates::{self, PageTemplate, PageTemplateBuilder},
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
use tracing::{debug, error, trace, warn};

pub type R<T> = core::result::Result<T, Error>;
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

pub fn get_markdown_contents(
    state: &AppState,
    rel_path: PathBuf,
) -> R<(PageTemplateBuilder<templates::Title>, String)> {
    let fs_path = state.root.join(rel_path).canonicalize()?;
    trace!(r#"Reading "{}""#, fs_path.display());
    let md = fs::read_to_string(&fs_path)?;
    trace!("Creating frontmatter exctractor");
    let mut extractor = FrontmatterExtractor::new(Parser::new_ext(&md, state.md_options));
    let mut content = String::new();
    trace!("Parsing markdown");
    pulldown_cmark::html::push_html(&mut content, &mut extractor);

    trace!("Parsing metadata");
    let toml = extractor
        .frontmatter
        .and_then(|fm| fm.code_block)
        .map(|cb| cb.source)
        .unwrap_or_else(|| {
            error!(r#"No fronmatter found!"#);
            r#"title = "Daniel's Website""#.into()
        });

    let metadata: Metadata = toml::from_str(&toml).unwrap_or_else(|err| {
        error!("Error parsing frontmatter: {err}");
        Metadata::default()
    });

    let l: OffsetDateTime = fs_path
        .metadata()
        .and_then(|md| md.modified())
        .map(|lm| lm.into())
        .unwrap_or_else(|err| {
            error!("Could not get last modified date: {err}");
            OffsetDateTime::now_utc()
        });
    Ok((
        PageTemplate::builder()
            .title(metadata.title)
            .last_modified(l.date())
            .tags_opt(metadata.tags),
        content,
    ))
}

pub fn render_markdown(State(state): State<AppState>, rel_path: PathBuf) -> R<Response> {
    debug!(r#"Serving markdown for "{}""#, rel_path.display());
    let (page, content) = get_markdown_contents(&state, rel_path)?;
    Ok(Html(
        page.build(state.root, content)?
            .render()
            .map_err(templates::Error::Template)?,
    )
    .into_response())
}

pub fn render_dir(State(state): State<AppState>, req_path: PathBuf) -> R<Response> {
    debug!(r#"Serving directory "{}""#, req_path.display());
    let req_path_fs = state.root.join(&req_path).canonicalize()?;
    // Filter out only valid files
    trace!("Formatting images");

    let mut sorted_entries = read_dir(state.root.join(&req_path))?
        .filter_map(Result::ok)
        .filter(|e| is_shown(e).unwrap_or(false))
        .map(get_paths(&state.root, &req_path))
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    // Sort
    sorted_entries.sort_by(|a, b| natord::compare(&a.display_name, &b.display_name));

    let (imgs, links) = sorted_entries
        .into_iter()
        .map(format_image_link(&state.root))
        // Separate any items which failed, just show link instead
        .partition_result();

    // Format links
    trace!("Formatting links");
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

    let l: OffsetDateTime = req_path_fs
        .metadata()
        .and_then(|md| md.modified())
        .map(|lm| lm.into())
        .unwrap_or_else(|err| {
            error!("Could not get last modified date: {err}");
            OffsetDateTime::now_utc()
        });
    Ok(Html(
        PageTemplate::builder()
            .title(&title)
            .last_modified(l.date())
            .build(
                state.root,
                format!(
                    r#"<div id="{}"><h1>{}</h1><div><div class="pic-grid">{}</div><div class="links"><ul class="links-list">{}</ul></div></div></div>"#,
                    req_path.display(),
                    title,
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
) -> impl FnMut(fs::DirEntry) -> R<Paths> + 'a {
    move |e| {
        trace!(r#"Getting paths for "{}""#, e.path().display());
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

fn format_image_link(root: &Path) -> impl FnMut(Paths) -> Result<String, Paths> + '_ {
    move |paths| {
        if !root.join(&paths.image_path).is_file() {
            trace!(r#"Could not find "{}""#, paths.image_path.display());
            return Err(paths);
        }
        trace!(r#"Formatting image for "{}""#, paths.entry_path.display());

        let caption = fs::read_to_string(&paths.description_path).unwrap_or_else(|err| {
            warn!(
                r#"No description found for "{}""#,
                paths.entry_path.display()
            );
            warn!(
                r#"Could not read "{}": {}"#,
                paths.description_path.display(),
                err
            );
            String::default()
        });
        let pg = PicGridTemplate {
            name: paths.display_name.clone(),
            img: format!("/{}", paths.image_path.display()),
            link: format!("/{}", paths.entry_path.with_extension("").display()),
            caption,
        };

        pg.render().map_err(|err| {
            warn!(
                r#"Could not render template for "{}": {}"#,
                paths.entry_path.display(),
                err
            );
            paths
        })
    }
}

fn format_links(paths: Paths) -> String {
    trace!(r#"Formatting link for "{}""#, paths.entry_path.display());
    format!(
        r#"<li><a href="/{}">{}</a></li>"#,
        paths.entry_path.with_extension("").display(),
        paths.display_name
    )
}
