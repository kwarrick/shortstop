use std::fmt;
use std::path::{Path, PathBuf};

use failure::{Backtrace, Context, Fail};

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        self.ctx.get_context()
    }
}

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
    Path(PathBuf),
    CommandLine(String),
}

impl ErrorKind {
    pub fn path<P: AsRef<Path>>(path: P) -> ErrorKind {
        ErrorKind::Path(path.as_ref().to_path_buf())
    }

    pub fn command(line: &str, error: structopt::clap::Error) -> ErrorKind {
        use structopt::clap::ErrorKind::*;

        let cmd = line
            .replacen("/", " ", 1)
            .split_whitespace()
            .next()
            .unwrap();

        let message = match error.kind {
            UnrecognizedSubcommand => {
                let cmd =
                    error.info.unwrap_or_default().pop().unwrap_or_default();
                format!(r#"Undefined command: "{}".  Try "help"."#, cmd)
            }
            _ => error.message,
        };

        ErrorKind::CommandLine(message)
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorKind::Path(ref path) => {
                //
                write!(f, "{}", path.display())
            }
            ErrorKind::CommandLine(ref s) => {
                //
                write!(f, "{}", s)
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
