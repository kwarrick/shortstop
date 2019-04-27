use std::fmt;
use std::path::{Path, PathBuf};

use failure::{Backtrace, Context, Fail};

pub type Result<T> = std::result::Result<T, Error>;

// impl Error {
//     pub fn kind(&self) -> &ErrorKind {
//         self.ctx.get_context()
//     }
// }

#[derive(Debug)]
pub struct Error {
    ctx: Context<ErrorKind>,
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.ctx.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.ctx.backtrace()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.ctx.fmt(f)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    /// Path related errors
    Path(PathBuf),
    /// Target program is not running
    NotRunning,
    /// Memory read error
    Read(usize),
    /// Memory write error
    Write(usize),
    /// Target debugger incompatible
    NotSupported,
}

impl ErrorKind {
    pub fn path<P: AsRef<Path>>(path: P) -> ErrorKind {
        ErrorKind::Path(path.as_ref().to_path_buf())
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorKind::Path(ref path) => {
                //
                write!(f, "{}", path.display())
            }
            ErrorKind::NotRunning => {
                //
                write!(f, "The program is not being run.")
            }
            ErrorKind::Read(addr) => {
                write!(f, "Cannot read memory at address 0x{:x}", addr)
            }
            ErrorKind::Write(addr) => {
                write!(f, "Cannot write memory at address 0x{:x}", addr)
            }
            ErrorKind::NotSupported => {
                write!(f, "Not supported on this target")
            }
        }
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error::from(Context::new(kind))
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(ctx: Context<ErrorKind>) -> Error {
        Error { ctx }
    }
}
