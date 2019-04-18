use std::ops::Deref;
use std::path::PathBuf;

use crate::{Binary, Debugger, Opt};

#[derive(Debug, Clone)]
struct Config {
    path: Option<PathBuf>,
    args: Vec<String>,
}

impl Config {
    fn new(opt: Opt) -> Self {
        Config {
            path: opt.prog,
            args: opt.args,
        }
    }
}

/// Environment covers an inner analysis state and applicaton configuration
#[derive(Debug)]
pub struct Env<T> {
    pub inner: T,
    config: Config,
}

impl Env<()> {
    // Build shortstop environment from command-line arguments
    pub fn new(opt: Opt) -> Self {
        Env {
            inner: (),
            config: Config::new(opt),
        }
    }
}

impl<T> Env<T> {
    pub fn set_path(&mut self, path: PathBuf) {
        self.config.path = Some(path);
    }

    /// Convert to debug context
    pub fn into_debugger(self, dbg: Debugger) -> Env<Debugger> {
        Env {
            inner: dbg,
            config: self.config,
        }
    }

    /// Convert to static object context
    pub fn into_binary(self, bin: Binary) -> Env<Binary> {
        Env {
            inner: bin,
            config: self.config,
        }
    }
}

impl<T> Deref for Env<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner
    }
}
