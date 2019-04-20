use std::path::PathBuf;

use failure::bail;

use super::*;

/// Handle "environment only" commands when no file has been specified
impl Env<()> {
    pub fn handle_command(&mut self, cmd: Cmd) -> Result<Option<Event>> {
        match cmd {
            Cmd::File { path } => self.handle_file_command(path),
            Cmd::Set { expr, cmd } => self.handle_set_command(expr, cmd),
            _ => bail!("No executable file specified."),
        }
    }

    fn handle_file_command(&mut self, path: PathBuf) -> Result<Option<Event>> {
        self.set_path(path.clone());
        let bin = Binary::new(path)?;
        Ok(Some(Event::Open(bin)))
    }

    fn handle_set_command(
        &mut self,
        expr: Option<String>,
        cmd: Option<Set>,
    ) -> Result<Option<Event>> {
        if expr.is_some() {
            bail!("set expression are not implemented yet");
        }

        match cmd {
            Some(Set::Args { args }) => self.handle_set_args(args),
            None => Ok(None),
        }
    }

    fn handle_set_args(&mut self, args: Vec<String>) -> Result<Option<Event>> {
        self.set_args(args);
        Ok(None)
    }
}
