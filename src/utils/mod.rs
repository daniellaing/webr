use std::{ffi::OsStr, path::PathBuf};

use crate::prelude::*;

pub trait PathBufExt {
    fn file_root(&self) -> Option<&str>;
}

impl PathBufExt for PathBuf {
    fn file_root(&self) -> Option<&str> {
        let fname = self.file_name().and_then(OsStr::to_str);
        fname
            .and_then(|s| s.split_once('.'))
            .and_then(|(before, _after)| Some(before))
            .or(fname)
    }
}
