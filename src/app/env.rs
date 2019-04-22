use std::ops::Deref;
use std::path::PathBuf;

use failure::bail;
use indexmap::IndexMap;

use super::*;

#[derive(Debug, Clone)]
struct Config {
    path: Option<PathBuf>,
    args: Vec<String>,
}

impl Config {
    fn new(opt: &Opt) -> Self {
        Config {
            path: opt.prog.clone(),
            args: opt.args.clone(),
        }
    }
}

/// Environment covers an inner analysis state and applicaton configuration
#[derive(Debug)]
pub struct Env<T> {
    pub inner: T,
    config: Config,
    breakpoints: IndexMap<usize, Breakpoint>,
    next_breakpoint_id: usize,
}

impl<T> Deref for Env<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner
    }
}

impl<T> Env<T> {
    /// Convert to debug context
    pub fn into_debugger(self, dbg: Debugger) -> Env<Debugger> {
        Env {
            inner: dbg,
            config: self.config,
            breakpoints: self.breakpoints,
            next_breakpoint_id: self.next_breakpoint_id,
        }
    }

    /// Convert to static object context
    pub fn into_binary(self, bin: Binary) -> Env<Binary> {
        Env {
            inner: bin,
            config: self.config,
            breakpoints: self.breakpoints,
            next_breakpoint_id: self.next_breakpoint_id,
        }
    }

    pub fn set_file(&mut self, path: PathBuf) -> Result<Option<Event>> {
        self.config.path = Some(path.clone());
        let bin = Binary::new(path)?;
        Ok(Some(Event::Open(bin)))
    }

    pub fn args(&self) -> Vec<String> {
        self.config.args.clone()
    }

    pub fn set_args(&mut self, args: Vec<String>) -> Result<Option<Event>> {
        self.config.args = args;
        Ok(None)
    }

    pub fn add_breakpoint(&mut self, loc: u64) -> Result<Option<Event>> {
        let breakpoint = Breakpoint::new(loc);
        self.breakpoints.insert(self.next_breakpoint_id, breakpoint);
        self.next_breakpoint_id += 1;
        Ok(None)
    }

    pub fn handle_set_command(
        &mut self,
        expr: Option<String>,
        cmd: Option<Set>,
    ) -> Result<Option<Event>> {
        if expr.is_some() {
            bail!("set expressions are not implemented yet");
        }

        match cmd {
            Some(Set::Args { args }) => self.set_args(args),
            None => Ok(None),
        }
    }
}

/// Handle "environment only" commands when no file has been specified
impl Env<()> {
    // Build shortstop environment from command-line arguments
    pub fn new(opt: &Opt) -> Self {
        Env {
            inner: (),
            config: Config::new(opt),
            breakpoints: IndexMap::new(),
            next_breakpoint_id: 0,
        }
    }

    pub fn handle_command(&mut self, cmd: Cmd) -> Result<Option<Event>> {
        match cmd {
            Cmd::File { path } => self.set_file(path),
            Cmd::Set { expr, cmd } => self.handle_set_command(expr, cmd),
            Cmd::Repeat => Ok(None),
            Cmd::Run { .. }
            | Cmd::Break { .. }
            | Cmd::Continue { .. }
            | Cmd::Examine { .. } => bail!("No executable file specified."),
        }
    }
}
