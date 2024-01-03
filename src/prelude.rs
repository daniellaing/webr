//! Crate prelude

pub use crate::error::Error;
pub type R<T> = core::result::Result<T, Error>;

pub use crate::app_state::AppState;
pub use pulldown_cmark::Options;
