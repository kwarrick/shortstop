use rustyline::error::ReadlineError;
use rustyline::Editor;
use structopt::StructOpt;

mod cli;
use cli::Opt;

mod dbg;
mod error;
use dbg::Debugger;

type Result<T> = std::result::Result<T, failure::Error>;

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

fn main() {
    let opt = Opt::from_args();
    if let Err(e) = prompt(opt) {
        eprintln!("{}", pretty_error(&e));
        std::process::exit(2);
    }
}

fn prompt(opt: Opt) -> Result<()> {
    let debugger = Debugger::new(opt.prog, opt.args)?;

    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline("(dbg) ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_ref());
                match cli::parse_command(&line) {
                    Ok(cmd) => debugger.exec(cmd),
                    Err(e) => println!("{}", e),
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
