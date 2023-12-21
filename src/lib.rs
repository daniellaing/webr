#![allow(unused)]

pub mod error;
pub mod prelude;
mod utils;

use std::collections::HashMap;

use crate::prelude::*;

pub fn init() -> Result<()> {
    env_logger::init();

    Ok(())
}

pub fn get_page(str: String) -> String {
    format!("Page placeholder.\n{str}")
}
