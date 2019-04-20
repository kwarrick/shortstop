use failure::bail;

use super::*;

impl Env<Debugger> {
    pub fn handle_command(&mut self, cmd: Cmd) -> Result<Option<Event>> {
        match cmd {
            Cmd::Run { args } => Ok(self.run_command(args)?),
            Cmd::Continue { n } => Ok(self.cont_command(n)?),
            Cmd::Examine { fmt, address } => {
                Ok(self.examine_command(fmt, address)?)
            }
            _ => bail!("not implemented"),
        }
    }

    fn run_command(&mut self, args: Vec<String>) -> Result<Option<Event>> {
        // Prompt to restart program
        println!("The program being debugged has been started already.");
        if cli::prompt_yes_no("Start it from the beginning?") {
            self.inner.run(args.to_vec())
        } else {
            println!("Program not restarted.");
        }
        Ok(None)
    }

    fn cont_command(&mut self, n: usize) -> Result<Option<Event>> {
        for _ in 0..n {
            self.inner.cont()
        }
        Ok(None)
    }

    fn examine_command(
        &mut self,
        fmt: Option<Fmt>,
        address: Option<u64>,
    ) -> Result<Option<Event>> {
        bail!("not implemented");
        Ok(None)
    }
}
