use failure::Error;
pub type Result<T> = std::result::Result<T, Error>;

use crate::cli::{self, Cmd, Fmt, Opt, Set};
use crate::dbg::{Address, Breakpoint, Debugger};
use crate::obj::Binary;

mod bin;
mod dbg;
mod env;
use env::Env;

/// Application contexts for command execution, for configuration before a
/// program is specified, "pure memory" analysis of a program before it is run,
/// and a context for analysis of a running debugged program.
#[derive(Debug)]
pub enum Context {
    Env(Env<()>),
    Static(Env<Binary>),
    Debug(Env<Debugger>),
}

/// Application context events
#[derive(Debug)]
pub enum Event {
    Open(Binary),
    Run(Debugger),
}

/// Application top-level
#[derive(Debug)]
pub struct Shortstop {
    ctx: Option<Context>,
}

impl Shortstop {
    pub fn new(opt: &Opt) -> Self {
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
            // Env --[ file ]--> Static
            (Context::Env(env), Some(Event::Open(bin))) => {
                Context::Static(env.into_binary(bin))
            }
            // Static --[ file ]--> Static
            (Context::Static(oldbin), Some(Event::Open(newbin))) => {
                Context::Static(oldbin.into_binary(newbin))
            }
            // Static --[ run ]--> Debug
            (Context::Static(bin), Some(Event::Run(dbg))) => {
                Context::Debug(bin.into_debugger(dbg))
            }
            // Debug --[ file ]--> Static
            (Context::Debug(dbg), Some(Event::Open(bin))) => {
                Context::Static(dbg.into_binary(bin))
            }
            (ctx, None) => ctx,
            _ => panic!("unhandled application event"),
        };

        self.ctx.replace(ctx);
        Ok(())
    }
}
