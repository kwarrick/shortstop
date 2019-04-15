use failure::Fail;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Fail, PartialEq)]
pub enum Error {
    #[fail(display = "{}", _0)]
    CommandLine(String),
}

impl Error {
    pub fn command(line: &str, error: structopt::clap::Error) -> Self {
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
            // MissingRequiredArgument => {
            //     let app = Cmd.clap();
            //     app.get_matches_from_safe(cmd)
            // }
            _ => error.message,
        };

        Error::CommandLine(message)
    }
}