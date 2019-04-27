use std::path::PathBuf;

use failure::{bail, ensure};
use structopt::{clap::AppSettings, StructOpt};

use super::Error;

/// Interactive prompt commands
#[derive(StructOpt, Debug)]
#[structopt(raw(setting = "AppSettings::SubcommandRequired"))]
#[structopt(raw(global_setting = "AppSettings::NoBinaryName"))]
#[structopt(raw(global_setting = "AppSettings::VersionlessSubcommands"))]
#[structopt(raw(global_setting = "AppSettings::InferSubcommands"))]
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
    #[structopt(raw(setting = "AppSettings::TrailingVarArg"))]
    Run {
        #[structopt(name = "ARGS")]
        args: Vec<String>,
    },
    #[structopt(
        name = "continue",
        about = "Continue program being debugged, after signal or breakpoint",
        template = "{bin} {positionals}"
    )]
    Continue {
        #[structopt(name = "N", default_value = "1")]
        n: usize,
    },
    #[structopt(
        name = "delete",
        about = "Delete some breakpoints",
        template = "{bin} {positionals}"
    )]
    Delete {
        #[structopt(name = "NUMS")]
        args: Vec<usize>,
    },
    #[structopt(
        name = "disable",
        about = "Disable some breakpoints.",
        template = "{bin} {positionals}"
    )]
    Disable {
        #[structopt(name = "NUMS")]
        args: Vec<usize>,
    },
    #[structopt(
        name = "break",
        about = "Set breakpoint at specified location",
        template = "{bin} {positionals}"
    )]
    Break {
        #[structopt(name = "LOCATION", parse(try_from_str = "parse_addr"))]
        loc: usize,
    },
    #[structopt(
        name = "x",
        template = "x/FMT ADDRESS",
        about = "Examine memory"
    )]
    #[structopt(raw(setting = "AppSettings::AllowLeadingHyphen"))]
    Examine {
        #[structopt(name = "FMT", parse(try_from_str = "parse_fmt"))]
        fmt: Option<Fmt>,
        #[structopt(name = "ADDRESS", parse(try_from_str = "parse_addr"))]
        addr: Option<usize>,
    },
    #[structopt(
        name = "file",
        template = "{bin} {positionals}",
        about = "Use file as program to be debugged"
    )]
    File {
        #[structopt(name = "FILE", parse(from_os_str))]
        path: PathBuf,
    },
    #[structopt(
        name = "set",
        template = "{usage}\n{subcommands}",
        about = "Commands that modify parts of the debug environment"
    )]
    #[structopt(raw(global_setting = "AppSettings::DisableHelpSubcommand"))]
    Set {
        expr: Option<String>,
        #[structopt(subcommand)]
        cmd: Option<Set>,
    },
    #[structopt(
        name = "info",
        template = "{subcommands}",
        about = "Generic command for showing things about the program being debugged"
    )]
    #[structopt(raw(global_setting = "AppSettings::DisableHelpSubcommand"))]
    Info {
        #[structopt(subcommand)]
        cmd: Info,
    },
}

/// Show subcommands for showing /proc information
#[derive(StructOpt, Debug)]
pub enum Info {
    #[structopt(
        name = "proc",
        template = "{subcommands}",
        about = "Show /proc process information about any running process"
    )]
    Proc {
        #[structopt(subcommand)]
        cmd: Proc,
    },
    #[structopt(
        name = "breakpoints",
        template = "{bin} {positionals}",
        about = "Status of specified breakpoints"
    )]
    Breakpoints {
        #[structopt(name = "NUM")]
        args: Vec<usize>,
    },
    #[structopt(
        name = "registers",
        template = "{bin} {positionals}",
        about = "List of integer registers and their contents"
    )]
    Registers {
        #[structopt(name = "NAMES")]
        names: Vec<String>,
    },
}

#[derive(StructOpt, Debug)]
pub enum Proc {
    #[structopt(name = "mappings", about = "List of mapped memory regions")]
    Mappings,
}

/// Set subcommands for configuring debugger environment settings
#[derive(StructOpt, Debug)]
pub enum Set {
    #[structopt(
        name = "args",
        about = "Set argument list to give program being debugged when it is started",
        template = "{bin} {positionals}"
    )]
    Args {
        #[structopt(name = "ARGS")]
        args: Vec<String>,
    },
}

/// Format for x, print, and display commands, i.e. x/FMT.
#[derive(Debug, Default, PartialEq)]
pub struct Fmt {
    pub reverse: bool,
    pub repeat: Option<u64>,
    pub format: Option<char>,
    pub size: Option<char>,
}

impl Fmt {
    pub fn update(&mut self, other: Self) {
        self.reverse = other.reverse;
        self.repeat = other.repeat.or(self.repeat);
        self.format = other.format.or(self.format);
        self.size = other.size.or(self.size);
    }
}

/// Parse a FMT for commands like x/FMT, e.g x/32wx
fn parse_fmt(arg: &str) -> Result<Fmt, failure::Error> {
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
            _ => bail!("Invalid output format: {}", c),
        }
    }
    if let Some(c) = letters.next() {
        match c {
            'b' | 'h' | 'w' | 'g' => fmt.size = Some(c),
            'o' | 'x' | 'd' | 'u' | 't' | 'f' | 'a' | 'i' | 'c' | 's' | 'z' => {
                fmt.format = Some(c)
            }
            _ => bail!("Invalid output size: {}", c),
        }
    }
    Ok(fmt)
}

/// Parse an address string
fn parse_addr(arg: &str) -> Result<usize, failure::Error> {
    ensure!(arg.len() > 0, "Cannot parse empty address string");
    let arg = arg.to_ascii_lowercase();
    if arg.starts_with("0x") {
        // Hex
        Ok(usize::from_str_radix(&arg[2..], 16)?)
    } else if arg.starts_with("0b") {
        // Binary
        Ok(usize::from_str_radix(&arg[2..], 2)?)
    } else if arg.starts_with("0o") {
        // Octal
        Ok(usize::from_str_radix(&arg[2..], 8)?)
    } else {
        // Decimal
        Ok(usize::from_str_radix(&arg, 10)?)
    }
}

/// Tokenize and parse a command line string
pub fn parse_command(line: &str) -> Result<Cmd, Error> {
    let cmd = match line.len() {
        0 => Ok(Cmd::Repeat),
        1 => Cmd::from_iter_safe(vec![line]),
        _ => {
            let line = match &line[..2] {
                "x/" | "p/" => line.replacen("/", " ", 1),
                _ => line.to_owned(),
            };
            Cmd::from_iter_safe(line.split_whitespace())
        }
    }
    .map_err(|e| Error::command(line, e))?;

    Ok(cmd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fmt() {
        assert_eq!(
            parse_fmt("32xw").ok(),
            Some(Fmt {
                reverse: false,
                repeat: Some(32),
                format: Some('x'),
                size: Some('w'),
            })
        );
        assert_eq!(
            parse_fmt("-32wx").ok(),
            Some(Fmt {
                reverse: true,
                repeat: Some(32),
                format: Some('x'),
                size: Some('w'),
            })
        );
    }

    #[test]
    fn test_parse_fmt_error() {
        assert!(parse_fmt("32kx").is_err());
        assert!(parse_fmt("32wk").is_err());
    }

    #[test]
    fn test_parse_addr() {
        assert_eq!(parse_addr("0x1000").ok(), Some(0x1000));
        assert_eq!(parse_addr("0b1010").ok(), Some(0b1010));
        assert_eq!(parse_addr("0o1111").ok(), Some(0o1111));
        assert_eq!(parse_addr("1234").ok(), Some(1234));
    }
}
