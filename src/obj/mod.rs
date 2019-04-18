use std::path::{Path, PathBuf};

use failure::Error;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Binary {
    pub path: PathBuf,
}

impl Binary {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().canonicalize()?;
        Ok(Binary { path })
    }
}
