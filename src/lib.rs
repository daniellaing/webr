#![allow(unused)]

pub mod error;
pub mod prelude;
mod utils;

use crate::prelude::*;

pub fn init() -> Result<()> {
    env_logger::init();

    Ok(())
}
