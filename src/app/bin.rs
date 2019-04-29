use failure::bail;

use super::*;

/// Handle "pure memory", static analysis
impl Env<Binary> {
    pub fn handle_command(&mut self, cmd: Cmd) -> Result<Option<Event>> {
        match cmd {
            Cmd::Break { loc } => self.break_command(loc),
            Cmd::Delete { args } => self.delete_command(args),
            Cmd::Disable { args } => self.disable_command(args),
            Cmd::Enable { args } => self.enable_command(args),
            Cmd::Examine { fmt, addr } => self.examine_command(fmt, addr),
            Cmd::File { path } => self.set_file(path),
            Cmd::Repeat => self.repeat_command(),
            Cmd::Run { args } => self.run_command(args),
            Cmd::Set { expr, cmd } => self.handle_set_command(expr, cmd),
            Cmd::Info { .. } | Cmd::Continue { .. } | Cmd::Stepi { .. } => {
                bail!("The program is not being run.")
            }
        }
    }

    fn break_command(&mut self, loc: usize) -> Result<Option<Event>> {
        self.add_breakpoint(loc);
        Ok(None)
    }

    fn delete_command(&mut self, args: Vec<usize>) -> Result<Option<Event>> {
        for num in args {
            if self.breakpoints.remove(&num).is_none() {
                println!("No breakpoint number {}.", num);
            }
        }
        Ok(None)
    }

    fn disable_command(&mut self, args: Vec<usize>) -> Result<Option<Event>> {
        for num in args {
            if let Some(bp) = self.breakpoints.get_mut(&num) {
                bp.enabled = false;
            }
        }
        Ok(None)
    }

    fn enable_command(&mut self, args: Vec<usize>) -> Result<Option<Event>> {
        for num in args {
            if let Some(bp) = self.breakpoints.get_mut(&num) {
                bp.enabled = true;
            }
        }
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
