use failure::bail;

use super::*;

/// Handle "pure memory", static analysis
impl Env<Binary> {
    pub fn handle_command(&mut self, cmd: Cmd) -> Result<Option<Event>> {
        match cmd {
            Cmd::Run { args } => Ok(self.run_command(args)?),
            _ => bail!("The program is not being run."),
        }
    }

    fn run_command(&mut self, args: Vec<String>) -> Result<Option<Event>> {
        let mut dbg = Debugger::new(&self.path)?;
        dbg.run(args);
        Ok(Some(Event::Run(dbg)))
    }
}
