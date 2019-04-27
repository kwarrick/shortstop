use failure::bail;

use std::path::PathBuf;

use super::*;

impl Env<Debugger> {
    pub fn handle_command(&mut self, cmd: Cmd) -> Result<Option<Event>> {
        match cmd {
            Cmd::Break { loc } => self.break_command(loc),
            Cmd::Continue { n } => self.continue_command(n),
            Cmd::Delete { args } => self.delete_command(args),
            Cmd::Disable { args } => self.disable_command(args),
            Cmd::Examine { fmt, addr } => self.examine_command(fmt, addr),
            Cmd::File { path } => self.file_command(path),
            Cmd::Repeat => self.repeat_command(),
            Cmd::Run { args } => self.run_command(args),
            Cmd::Set { expr, cmd } => self.handle_set_command(expr, cmd),
            Cmd::Info { cmd } => self.info_command(cmd),
        }
    }

    fn break_command(&mut self, addr: usize) -> Result<Option<Event>> {
        let num = self.add_breakpoint(addr);
        self.breakpoints
            .get_mut(&num)
            .unwrap()
            .enable(&mut self.inner)?;
        Ok(None)
    }

    fn continue_command(&mut self, n: usize) -> Result<Option<Event>> {
        for _ in 0..n {
            self.inner.cont()?
        }
        Ok(None)
    }

    fn delete_command(&mut self, args: Vec<usize>) -> Result<Option<Event>> {
        if args.is_empty() {
            if cli::prompt_yes_no("Delete all breakpoints?") {
                for (_, mut bp) in self.breakpoints.drain(..) {
                    bp.disable(&mut self.inner)?;
                }
            }
        } else {
            for num in args {
                match self.breakpoints.remove(&num) {
                    Some(mut bp) => bp.disable(&mut self.inner)?,
                    None => println!("No breakpoint number {}.", num),
                }
            }
        }
        Ok(None)
    }

    fn disable_command(&mut self, args: Vec<usize>) -> Result<Option<Event>> {
        for num in args {
            match self.breakpoints.get_mut(&num) {
                Some(mut bp) => bp.disable(&mut self.inner)?,
                None => println!("No breakpoint number {}.", num),
            }
        }
        Ok(None)
    }

    fn examine_command(
        &mut self,
        fmt: Option<Fmt>,
        addr: Option<usize>,
    ) -> Result<Option<Event>> {
        // Update last used format and address
        if let Some(fmt) = fmt {
            self.set_fmt(fmt);
        }
        if let Some(addr) = addr {
            self.set_addr(addr);
        }

        let mut addr = match self.addr() {
            Some(addr) => addr,
            None => bail!("Argument required (starting display address)."),
        };

        // Unwrap format with defaults
        let fmt = self.fmt();
        let reverse = fmt.reverse;
        let repeat = fmt.repeat.unwrap_or(1);
        let size = fmt.size.unwrap_or('w');
        let format = fmt.format.unwrap_or('x');

        // Convert size char to byte size and column count
        let (step, size) = match size {
            'b' => (8, 1),
            'h' => (8, 2),
            'w' => (4, 4),
            'g' => (2, 8),
            _ => unreachable!(),
        };

        let mut i = 0;
        while i < repeat {
            let mut j = 0;
            print!("{:#x}: ", addr);
            while j < step && (i + j) < repeat {
                // Read bytes
                let mut bytes = [0; 8];

                self.inner
                    .read(addr, size)?
                    .iter()
                    .enumerate()
                    .for_each(|(i, byte)| bytes[i] = *byte);

                match format {
                    'x' => print!(
                        " {:0width$x}",
                        usize::from_le_bytes(bytes),
                        width = size * 2
                    ),
                    _ => unimplemented!(),
                }

                j += 1;
                addr += size;
            }
            println!();
            i += j;
        }

        self.set_addr(addr);
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

    fn info_command(&mut self, cmd: cli::Info) -> Result<Option<Event>> {
        match cmd {
            cli::Info::Proc { cmd } => self.info_proc_command(cmd)?,
            cli::Info::Breakpoints { args } => {
                self.info_breakpoints_command(args)?
            }
        };
        Ok(None)
    }

    fn info_proc_command(&mut self, cmd: cli::Proc) -> Result<Option<Event>> {
        let proc = self.inner.proc()?;
        match cmd {
            cli::Proc::Mappings => {
                dbg!(proc.proc_maps());
            }
        }
        Ok(None)
    }

    fn info_breakpoints_command(
        &mut self,
        args: Vec<usize>,
    ) -> Result<Option<Event>> {
        for bp in self.breakpoints.iter() {
            dbg!(bp);
        }
        Ok(None)
    }
}
