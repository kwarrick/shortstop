use failure::bail;
use structopt::{clap::AppSettings, StructOpt};

use super::Error;

/// Interactive prompt commands
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
    #[structopt(raw(setting = "AppSettings::TrailingVarArg"))]
    Run {
        #[structopt(name = "ARGS")]
        args: Vec<String>,
    },
    #[structopt(
        name = "continue",
        about = "Continue program being debugged, after signal or breakpoint.",
        template = "{bin} {positionals}"
    )]
    Continue {
        #[structopt(name = "N", default_value = "1")]
        n: usize,
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
