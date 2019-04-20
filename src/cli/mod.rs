use rustyline::{error::ReadlineError, Editor};
use structopt::StructOpt;

mod error;
pub use error::Error;

mod opt;
pub use opt::Opt;

mod cmd;
pub use cmd::{parse_command, Cmd, Fmt, Set};

pub fn prompt_yes_no<P: AsRef<str>>(prompt: P) -> bool {
    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline(&format!("{} (y or n) ", prompt.as_ref()));
        match readline {
            Ok(ref line) if line.len() > 0 => {
                match line.chars().next().unwrap() {
                    'y' | 'Y' => break true,
                    'n' | 'N' => break false,
                    _ => {
                        println!("Please answer y or n");
                        continue;
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Quit");
                break false;
            }
            Err(ReadlineError::Eof) => {
                println!("EOF [assumed Y]");
                break true;
            }
            Err(err) => {
                println!("error: {:?}", err);
                break false;
            }
            _ => continue,
        }
    }
}
