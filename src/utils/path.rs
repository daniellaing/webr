use std::path::{Path, PathBuf};

pub trait PathExt {
    fn file_root(&self) -> Option<&str>;
    fn is_hidden(&self) -> Option<bool>;
}

impl PathExt for Path {
    fn file_root(&self) -> Option<&str> {
        let fname = self.file_name().and_then(std::ffi::OsStr::to_str);
        fname
            .and_then(|s| s.split_once('.'))
            .map(|(before, _after)| before)
            .or(fname)
    }

    fn is_hidden(&self) -> Option<bool> {
        self.file_name()
            .map(std::ffi::OsStr::to_string_lossy)
            .map(|s| s.starts_with('.'))
    }
}
