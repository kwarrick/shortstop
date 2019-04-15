use std::num::ParseIntError;
use std::path::PathBuf;

use failure::Fail;
use structopt::clap::AppSettings;
use structopt::StructOpt;

/// Command line options
#[derive(StructOpt, Debug)]
pub struct Opt {
    #[structopt(name = "PROG", parse(from_os_str))]
    pub prog: PathBuf,
    #[structopt(name = "ARGS")]
    pub args: Vec<String>,
}

/// Interactive commands
#[derive(StructOpt, Debug)]
#[structopt(raw(setting = "AppSettings::NoBinaryName"))]
#[structopt(raw(setting = "AppSettings::VersionlessSubcommands"))]
#[structopt(raw(setting = "AppSettings::InferSubcommands"))]
#[structopt(raw(setting = "AppSettings::SubcommandRequired"))]
#[structopt(raw(global_setting = "AppSettings::DontCollapseArgsInUsage"))]
#[structopt(template = "{subcommands}")]
pub enum Cmd {
    #[structopt(raw(setting = "AppSettings::Hidden"))]
    Repeat,
    #[structopt(
        name = "run",
        about = "Start debugged program",
        template = "{bin} {positionals}"
    )]
    Run {
        #[structopt(name = "ARGS")]
        args: Vec<String>,
    },
    #[structopt(
        name = "break",
        about = "Set breakpoint at specified location.",
        template = "{bin} {positionals}"
    )]
    Break {
        #[structopt(name = "LOCATION")]
        loc: u64,
    },
    #[structopt(
        name = "x",
        template = "x/FMT ADDRESS",
        about = "Examine memory."
    )]
    #[structopt(raw(setting = "AppSettings::AllowLeadingHyphen"))]
    Examine {
        #[structopt(name = "FMT", parse(try_from_str = "parse_fmt"))]
        fmt: Option<Fmt>,
        #[structopt(name = "ADDRESS")]
        address: Option<u64>,
    },
}

/// Format for x, print, and display commands, i.e. x/FMT.
#[derive(Debug, Default, PartialEq)]
pub struct Fmt {
    reverse: bool,
    repeat: Option<u64>,
    format: Option<char>,
    size: Option<char>,
}

#[derive(Debug, Fail, PartialEq)]
enum FmtError {
    #[fail(display = "Undefined output format \"{}\"", _0)]
    InvalidOutputFormat(char),
    #[fail(display = "Undefined output size \"{}\"", _0)]
    InvalidOutputSize(char),
    #[fail(display = "Undefined repeat count \"{}\"", _0)]
    InvalidRepeatCount(#[fail(cause)] ParseIntError),
}

impl std::convert::From<std::num::ParseIntError> for FmtError {
    fn from(error: ParseIntError) -> FmtError {
        FmtError::InvalidRepeatCount(error)
    }
}

/// Parse a FMT for commands like x/FMT, e.g x/32wx
fn parse_fmt(arg: &str) -> Result<Fmt, FmtError> {
    let mut fmt: Fmt = Default::default();

    let mut s = arg;
    if s.starts_with("-") {
        fmt.reverse = true;
        s = s.trim_start_matches("-");
    }

    let repeat: String = s
        .chars()
        .take_while(|b| b.is_digit(10))
        .into_iter()
        .collect();

    if repeat.len() > 0 {
        fmt.repeat = Some(repeat.parse::<u64>()?);
        s = s.trim_start_matches(char::is_numeric);
    }

    let mut letters = s.chars().take(2);
    if let Some(c) = letters.next() {
        match c {
            'b' | 'h' | 'w' | 'g' => fmt.size = Some(c),
            'o' | 'x' | 'd' | 'u' | 't' | 'f' | 'a' | 'i' | 'c' | 's' | 'z' => {
                fmt.format = Some(c)
            }
            _ => return Err(FmtError::InvalidOutputFormat(c)),
        }
    }
    if let Some(c) = letters.next() {
        match c {
            'b' | 'h' | 'w' | 'g' => fmt.size = Some(c),
            'o' | 'x' | 'd' | 'u' | 't' | 'f' | 'a' | 'i' | 'c' | 's' | 'z' => {
                fmt.format = Some(c)
            }
            _ => return Err(FmtError::InvalidOutputSize(c)),
        }
    }
    Ok(fmt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fmt() {
        assert_eq!(
            parse_fmt("32xw"),
            Ok(Fmt {
                reverse: false,
                repeat: Some(32),
                format: Some('x'),
                size: Some('w'),
            })
        );
        assert_eq!(
            parse_fmt("-32wx"),
            Ok(Fmt {
                reverse: true,
                repeat: Some(32),
                format: Some('x'),
                size: Some('w'),
            })
        );
    }

    #[test]
    fn test_parse_fmt_error() {
        assert_eq!(parse_fmt("32kx"), Err(FmtError::InvalidOutputFormat('k')));
        assert_eq!(parse_fmt("32wk"), Err(FmtError::InvalidOutputSize('k')));
    }
}
