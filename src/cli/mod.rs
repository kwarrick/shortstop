use rustyline::{error::ReadlineError, Editor};
use structopt::StructOpt;

mod error;
use error::Error;

mod opt;
pub use opt::Opt;

mod cmd;
pub use cmd::{parse_command, Cmd, Fmt};

pub fn prompt_yes_no<P: AsRef<str>>(prompt: P) -> bool {
    let mut rl = Editor::<()>::new();
    let readline = rl.readline(&format!("{} (y or n) ", prompt.as_ref()));
    loop {
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
}
