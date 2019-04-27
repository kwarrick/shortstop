use failure::bail;

use super::*;

/// Handle "pure memory", static analysis
impl Env<Binary> {
    pub fn handle_command(&mut self, cmd: Cmd) -> Result<Option<Event>> {
        match cmd {
            Cmd::Break { loc } => self.break_command(loc),
            Cmd::Examine { fmt, addr } => self.examine_command(fmt, addr),
            Cmd::File { path } => self.set_file(path),
            Cmd::Repeat => self.repeat_command(),
            Cmd::Run { args } => self.run_command(args),
            Cmd::Set { expr, cmd } => self.handle_set_command(expr, cmd),
            Cmd::Info { .. } | Cmd::Continue { .. } => {
                bail!("The program is not being run.")
            }
        }
    }

    fn break_command(&mut self, loc: usize) -> Result<Option<Event>> {
        self.add_breakpoint(loc);
        Ok(None)
    }

    fn examine_command(
        &mut self,
        fmt: Option<Fmt>,
        addr: Option<usize>,
    ) -> Result<Option<Event>> {
        dbg!((fmt, addr));
        Ok(None)
    }

    fn repeat_command(&mut self) -> Result<Option<Event>> {
        Ok(None)
    }

    fn run_command(&mut self, args: Vec<String>) -> Result<Option<Event>> {
        if args.len() > 0 {
            self.set_args(args)?;
        }
        let mut dbg = Debugger::new(&self.path)?;
        dbg.run(self.args());
        Ok(Some(Event::Run(dbg)))
    }
}
