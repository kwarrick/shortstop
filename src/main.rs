use std::path::{Path, PathBuf};

use rustyline::{error::ReadlineError, Editor};
use structopt::StructOpt;

mod app;
use app::Shortstop;

mod cli;
use cli::{Cmd, Opt};

mod dbg;
use dbg::Debugger;

mod obj;
use obj::Binary;

use failure::Error;
pub type Result<T> = std::result::Result<T, Error>;

fn main() {
    let opt = Opt::from_args();
    if let Err(e) = command_prompt(opt) {
        eprintln!("{}", pretty_error(&e));
        std::process::exit(2);
    }
}

fn command_prompt(opt: Opt) -> Result<()> {
    let mut shortstop = Shortstop::new(opt);

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
