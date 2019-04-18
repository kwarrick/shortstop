use std::cell::Cell;
use std::fmt::Debug;
use std::path::PathBuf;

use failure::bail;

use failure::Error;
pub type Result<T> = std::result::Result<T, Error>;

use crate::{cli, cli::Fmt, Binary, Cmd, Debugger, Env, Opt};

#[derive(Debug)]
enum Event {
    Open(Binary),
    Run(Debugger),
}

#[derive(Debug)]
enum Context {
    Env(Env<()>),
    Static(Env<Binary>),
    Debug(Env<Debugger>),
}

/// Application top-level
#[derive(Debug)]
pub struct Shortstop {
    ctx: Option<Context>,
}

/// Handle "environment only" commands when no file has been specified
impl Env<()> {
    fn handle_command(&mut self, cmd: Cmd) -> Result<Option<Event>> {
        match cmd {
            Cmd::File { path } => {
                self.set_path(path.clone());
                let bin = Binary::new(path)?;
                Ok(Some(Event::Open(bin)))
            }
            _ => bail!("No executable file specified."),
        }
    }
}

/// Handle "pure memory", static analysis
impl Env<Binary> {
    fn handle_command(&mut self, cmd: Cmd) -> Result<Option<Event>> {
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

impl Env<Debugger> {
    fn handle_command(&mut self, cmd: Cmd) -> Result<Option<Event>> {
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

impl Shortstop {
    pub fn new(opt: Opt) -> Self {
        Shortstop {
            ctx: Some(Context::Env(Env::new(opt))),
        }
    }

    pub fn handle_command_line(&mut self, line: String) -> Result<()> {
        let cmd = cli::parse_command(&line)?;
        self.handle_command(cmd)
    }

    pub fn handle_command(&mut self, cmd: Cmd) -> Result<()> {
        let ctx = self.ctx.take();

        // Context-sensitive command dispatch
        let (ctx, event) = match ctx {
            Some(Context::Env(mut env)) => {
                let event = env.handle_command(cmd);
                (Context::Env(env), event)
            }
            Some(Context::Static(mut bin)) => {
                let event = bin.handle_command(cmd);
                (Context::Static(bin), event)
            }
            Some(Context::Debug(mut dbg)) => {
                let event = dbg.handle_command(cmd);
                (Context::Debug(dbg), event)
            }
            None => panic!("application context is none"),
        };

        // Unwrap result or restore context and return error
        let event = match event {
            Ok(event) => event,
            Err(e) => {
                self.ctx.replace(ctx);
                return Err(e);
            }
        };

        // Event-based application context switches
        let ctx = match (ctx, event) {
            // Env -- open file --> Static
            (Context::Env(env), Some(Event::Open(bin))) => {
                Context::Static(env.into_binary(bin))
            }
            // Static -- open file --> Static
            (Context::Static(oldbin), Some(Event::Open(newbin))) => {
                Context::Static(oldbin.into_binary(newbin))
            }
            // Static -- run --> Debug
            (Context::Static(bin), Some(Event::Run(dbg))) => {
                Context::Debug(bin.into_debugger(dbg))
            }
            (ctx, None) => ctx,
            _ => panic!("unhandled application event"),
        };

        self.ctx.replace(ctx);
        Ok(())
    }
}
