#![allow(unused)]
#![recursion_limit = "512"]

pub mod error;
pub mod prelude;
mod utils;

use html::{content::Footer, root::Html};

use crate::prelude::*;

pub fn init() -> Result<()> {
    env_logger::init();

    Ok(())
}

pub fn get_page() -> String {
    utils::html::get_page().to_string()
}

pub fn create_footer<A, S>(email: S, items: A) -> Footer
where
    A: IntoIterator<Item = (S, S)>,
    S: Into<std::borrow::Cow<'static, str>> + std::fmt::Display,
{
    utils::html::footer(email, items)
}
