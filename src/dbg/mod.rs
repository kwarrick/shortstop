use std::fmt::Debug;
use std::path::{Path, PathBuf};

use failure::ResultExt;

mod error;
pub use error::{Error, ErrorKind, Result};

mod ptrace;
use ptrace::Ptraced;

/// Debugger with generic debugged progam type
#[derive(Debug)]
pub struct Debugger {
    prog: PathBuf,
    debugged: Option<Box<dyn Debugged>>,
}

/// Generic debugged program type
pub trait Debugged: Debug {
    /// Start debugged program
    fn run(&mut self, args: Vec<String>);
    /// Read from memory of debugged program
    fn read(&mut self, vaddr: u64, size: usize) -> Result<Vec<u8>>;
    /// Write to memory of debugged program
    fn write(&mut self, vaddr: u64, data: &[u8]) -> Result<usize>;
    /// Continue program execution
    fn cont(&mut self) -> Result<()>;
    /// Step one instruction exactly
    fn step(&mut self, count: usize);
}

/// Interactive debugger type
impl Debugger {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let prog = path
            .as_ref()
            .canonicalize()
            .with_context(|_| ErrorKind::path(&path))?;

        Ok(Debugger {
            prog,
            debugged: None,
        })
    }

    /// Set a soft breakpoint and return the replaced byte
    pub fn set_breakpoint(&self, vaddr: u64) -> Result<u8> {
        unimplemented!()
    }

    /// Remove a soft breakpoint, restore saved byte
    pub fn remove_breakpoint(&self, vaddr: u64, saved: u8) -> Result<()> {
        unimplemented!()
    }

    pub fn read(&mut self, vaddr: u64, n: usize) -> Result<Vec<u8>> {
        let mut target = self.debugged.as_mut().ok_or(ErrorKind::NotRunning)?;
        target.read(vaddr, n)
    }

    pub fn cont(&mut self) -> Result<()> {
        let mut target = self.debugged.as_mut().ok_or(ErrorKind::NotRunning)?;
        target.cont()
    }

    pub fn run(&mut self, args: Vec<String>) {
        println!(
            "Starting program: {} {}",
            self.prog.display(),
            args.join(" "),
        );

        self.debugged = Some(Ptraced::new(self.prog.clone()));
        if let Some(target) = self.debugged.as_mut() {
            target.run(args.clone());
        }
    }
}

/// Soft breakpoint type
#[derive(Debug)]
pub struct Breakpoint {
    addr: u64,
    enabled: bool,
    saved: Option<u8>,
}

impl Breakpoint {
    pub fn new(addr: u64) -> Self {
        Breakpoint {
            addr,
            enabled: false,
            saved: None,
        }
    }

    fn enable(&mut self, dbg: Debugger) -> Result<()> {
        self.saved.replace(dbg.set_breakpoint(self.addr)?);
        Ok(())
    }

    fn disable(&mut self, dbg: Debugger) -> Result<()> {
        if let Some(byte) = self.saved {
            dbg.remove_breakpoint(self.addr, byte)?;
        }
        Ok(())
    }
}
