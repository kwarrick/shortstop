use std::path::{Path, PathBuf};

use failure::bail;
use rustyline::{error::ReadlineError, Editor};
use structopt::StructOpt;

mod cli;
use cli::{Cmd, Opt};

mod dbg;
use dbg::Debugger;

mod obj;
use obj::Binary;

use failure::Error;
pub type Result<T> = std::result::Result<T, Error>;

/// Simple state machine over analysis contexts
enum State {
    Static(Binary),
    Running(Debugger),
}

/// Application top-level
struct Shortstop {
    path: Option<PathBuf>,
    args: Vec<String>,
    state: Option<State>,
}

/// Handle "pure memory", static analysis
impl Binary {
    fn handle_command(self, cmd: Cmd) -> Result<State> {
        match cmd {
            Cmd::Run { args } => self.run_command(args),
            _ => bail!("The program is not being run."),
        }
    }

    fn run_command(self, args: Vec<String>) -> Result<State> {
        let mut dbg = Debugger::new(self.path)?;
        dbg.run(args);
        Ok(State::Running(dbg))
    }
}

/// Handle commands for a running debugged process
impl Debugger {
    fn handle_command(mut self, cmd: Cmd) -> Result<State> {
        match cmd {
            Cmd::Run { args } => self.run_command(args),
            Cmd::Continue { n } => self.cont_command(n),
            _ => unimplemented!(),
        }
    }

    fn run_command(mut self, args: Vec<String>) -> Result<State> {
        println!("The program being debugged has been started already.");
        if cli::prompt_yes_no("Start it from the beginning?") {
            self.run(args.to_vec())
        } else {
            println!("Program not restarted.");
        }
        Ok(State::Running(self))
    }

    fn cont_command(mut self, n: usize) -> Result<State> {
        for _ in 0..n {
            self.cont()
        }
        Ok(State::Running(self))
    }
}

impl Shortstop {
    fn new(path: Option<PathBuf>, args: Vec<String>) -> Result<Self> {
        Ok(Shortstop {
            path,
            args,
            state: None,
        })
    }

    fn handle_command_line(&mut self, line: String) -> Result<()> {
        let cmd = cli::parse_command(&line)?;
        let state = match self.state.take() {
            Some(State::Static(bin)) => bin.handle_command(cmd)?,
            Some(State::Running(dbg)) => dbg.handle_command(cmd)?,
            None => self.handle_command(cmd)?,
        };
        self.state.replace(state);
        Ok(())
    }

    /// Handle commands when no file has been specified
    fn handle_command(&mut self, cmd: Cmd) -> Result<State> {
        match cmd {
            Cmd::File { path } => Ok(State::Static(Binary::new(path)?)),
            _ => bail!("No executable file specified."),
        }
    }

    // Cmd::Break { loc } => debugger.breakpoint(loc),
    // Cmd::Examine { fmt, address } => unimplemented!(),
    // Cmd::Repeat => debugger.repeat(),
    // Cmd::Run { args } => debugger.run(args),
}

fn main() {
    let opt = Opt::from_args();
    if let Err(e) = command_prompt(opt) {
        eprintln!("{}", pretty_error(&e));
        std::process::exit(2);
    }
}

fn command_prompt(opt: Opt) -> Result<()> {
    let mut shortstop = Shortstop::new()?;

    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline("(dbg) ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_ref());
                if let Err(e) = shortstop.handle_command_line(line) {
                    println!("{}", e);
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Quit");
            }
            Err(ReadlineError::Eof) => {
                println!("quit");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}

/// Return a prettily formatted error, including its entire causal chain.
/// credit: https://github.com/BurntSushi/
fn pretty_error(err: &failure::Error) -> String {
    let mut pretty = err.to_string();
    let mut prev = err.as_fail();
    while let Some(next) = prev.cause() {
        pretty.push_str(": ");
        pretty.push_str(&next.to_string());
        prev = next;
    }
    pretty
}
