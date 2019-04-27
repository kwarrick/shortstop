use std::fmt::Debug;
use std::ops::Range;
use std::path::PathBuf;

use super::{ErrorKind, Result};

#[derive(Debug)]
pub struct Map {
    pub address_range: Range<usize>,
    pub perms: String,
    pub read: bool,
    pub write: bool,
    pub execute: bool,
    pub private: bool,
    pub offset: usize,
    pub device_major: u16,
    pub device_minor: u16,
    pub inode: usize,
    pub pathname: PathBuf,
}

pub trait Proc: Debug {
    fn proc(&self) -> Box<dyn ProcReader>;
}

/// Read /proc process information a process by PID
pub trait ProcReader: Debug {
    fn proc_maps(&self) -> Result<Vec<Map>>;
}

// /// Default implementation returns NotSupported errors
// #[derive(Debug)]
// pub struct NotSupported;

// impl ProcReader for NotSupported {
//     fn proc_maps(&self) -> Result<Vec<Map>> {
//         Err(ErrorKind::NotSupported.into())
//     }
// }

// impl Proc for SomeTarget {
//     fn proc(&self) -> Box<dyn ProcReader> {
//         Box::new(proc::NotSupported)
//     }
// }
