use failure::bail;

use super::*;

/// Handle "pure memory", static analysis
impl Env<Binary> {
    pub fn handle_command(&mut self, cmd: Cmd) -> Result<Option<Event>> {
        match cmd {
            Cmd::Break { loc } => Ok(None),
            Cmd::Continue { n } => bail!("The program is not being run."),
            Cmd::Examine { fmt, addr } => Ok(None),
            Cmd::File { path } => Ok(None),
            Cmd::Repeat => Ok(None),
            Cmd::Run { args } => self.run_command(args),
            Cmd::Set { expr, cmd } => self.handle_set_command(expr, cmd),
        }
    }

    fn run_command(&mut self, args: Vec<String>) -> Result<Option<Event>> {
        let mut dbg = Debugger::new(&self.path)?;
        dbg.run(args);
        Ok(Some(Event::Run(dbg)))
    }
}
