#![allow(unused)]
#![recursion_limit = "512"]

pub mod error;
pub mod prelude;
mod utils;

use std::collections::HashMap;

use html::{content::Footer, root::Html};

use crate::prelude::*;

pub struct Document {
    markdown: String,
    metadata: Metadata,
}

pub struct Metadata {
    title: String,
    socials: HashMap<String, String>,
}

pub fn init() -> Result<()> {
    env_logger::init();

    Ok(())
}

pub fn get_page(doc: Document) -> String {
    utils::html::get_page(doc).to_string()
}
