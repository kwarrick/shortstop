use failure::bail;

use std::path::PathBuf;

use super::*;

impl Env<Debugger> {
    pub fn handle_command(&mut self, cmd: Cmd) -> Result<Option<Event>> {
        match cmd {
            Cmd::Break { loc } => self.break_command(loc),
            Cmd::Continue { n } => self.continue_command(n),
            Cmd::Examine { fmt, addr } => self.examine_command(fmt, addr),
            Cmd::File { path } => self.file_command(path),
            Cmd::Repeat => self.repeat_command(),
            Cmd::Run { args } => self.run_command(args),
            Cmd::Set { expr, cmd } => self.handle_set_command(expr, cmd),
        }
    }

    fn break_command(&mut self, addr: u64) -> Result<Option<Event>> {
        self.inner.set_breakpoint(addr as Address);
        Ok(None)
    }

    fn continue_command(&mut self, n: usize) -> Result<Option<Event>> {
        for _ in 0..n {
            self.inner.cont()?
        }
        Ok(None)
    }

    fn examine_command(
        &mut self,
        fmt: Option<Fmt>,
        addr: Option<u64>,
    ) -> Result<Option<Event>> {
        dbg!(fmt);
        dbg!(addr);
        if let Some(vaddr) = addr {
            let data = self.inner.read(vaddr as Address, 1);
            dbg!(data);
        }
        Ok(None)
    }

    fn file_command(&mut self, path: PathBuf) -> Result<Option<Event>> {
        println!("A program is being debugged already.");
        if cli::prompt_yes_no("Are you sure you want to change the file?") {
            self.set_file(path)
        } else {
            println!("File not changed.");
            Ok(None)
        }
    }

    fn run_command(&mut self, args: Vec<String>) -> Result<Option<Event>> {
        // Prompt to restart program
        println!("The program being debugged has been started already.");
        if cli::prompt_yes_no("Start it from the beginning?") {
            if args.len() > 0 {
                self.set_args(args)?;
            }
            self.inner.run(self.args())
        } else {
            println!("Program not restarted.");
        }
        Ok(None)
    }

    fn repeat_command(&mut self) -> Result<Option<Event>> {
        Ok(None)
    }
}
