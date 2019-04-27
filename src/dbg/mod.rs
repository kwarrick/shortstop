use std::fmt::Debug;
use std::path::{Path, PathBuf};

use failure::ResultExt;

mod error;
pub use error::{Error, ErrorKind, Result};

mod proc;
use proc::{Proc, ProcReader};

mod ptrace;
use ptrace::Ptraced;

pub type Address = usize;

/// Debugger with generic debugged progam type
#[derive(Debug)]
pub struct Debugger {
    prog: PathBuf,
    target: Option<Box<dyn Target>>,
}

/// Generic debugged program interface
pub trait Debugged: Debug {
    /// Start debugged program
    fn run(&mut self, args: Vec<String>);
    /// Read from memory of debugged program
    fn read(&mut self, vaddr: Address, size: usize) -> Result<Vec<u8>>;
    /// Write to memory of debugged program
    fn write(&mut self, vaddr: Address, data: &[u8]) -> Result<usize>;
    /// Continue program execution
    fn cont(&mut self) -> Result<()>;
    /// Step one instruction exactly
    fn step(&mut self, count: usize);
}

/// Target is the common interface for a heterogenous set of traits
pub trait Target: Debugged + Proc {
    // empty
}

/// Interactive debugger type
impl Debugger {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let prog = path
            .as_ref()
            .canonicalize()
            .with_context(|_| ErrorKind::path(&path))?;

        Ok(Debugger { prog, target: None })
    }

    /// Return mutable reference to inner debugged type
    fn target(&mut self) -> Result<&mut Box<Target>> {
        Ok(self.target.as_mut().ok_or(ErrorKind::NotRunning)?)
    }

    /// Set a soft breakpoint and return the replaced byte
    pub fn set_breakpoint(&mut self, vaddr: Address) -> Result<u8> {
        let target = self.target()?;
        // Read byte at address
        let bytes = target.read(vaddr, 1)?;
        // Write int3 for soft breakpoint
        target.write(vaddr, &vec![0xCC])?;
        Ok(bytes[0])
    }

    /// Remove a soft breakpoint, restore saved byte
    pub fn remove_breakpoint(&self, vaddr: Address, saved: u8) -> Result<()> {
        unimplemented!()
    }

    /// Read from memory of debugged process
    pub fn read(&mut self, vaddr: Address, n: usize) -> Result<Vec<u8>> {
        self.target()?.read(vaddr, n)
    }

    /// Write to memory of debugged process
    pub fn write(&mut self, vaddr: Address, data: &[u8]) -> Result<()> {
        self.target()?.write(vaddr, data).map(|_| ())
    }

    /// Continue execution of debugged process
    pub fn cont(&mut self) -> Result<()> {
        self.target()?.cont()
    }

    /// Run a new debugged process
    pub fn run(&mut self, args: Vec<String>) {
        println!(
            "Starting program: {} {}",
            self.prog.display(),
            args.join(" "),
        );

        self.target = Some(Ptraced::new(self.prog.clone()));
        if let Some(target) = self.target.as_mut() {
            target.run(args.clone());
        }
    }

    /// Return a /proc reader
    pub fn proc(&mut self) -> Result<Box<dyn ProcReader>> {
        Ok(self.target()?.proc())
    }
}

/// Soft breakpoint type
#[derive(Debug)]
pub struct Breakpoint {
    /// Target virtual address
    addr: Address,
    /// Breakpoint active flag
    enabled: bool,
    /// Saved instruction byte
    saved: Option<u8>,
}

impl Breakpoint {
    /// Create a new breakpoint for a target address
    pub fn new(addr: Address) -> Self {
        Breakpoint {
            addr,
            enabled: false,
            saved: None,
        }
    }

    /// Enable breakpoint on debugger
    fn enable(&mut self, dbg: &mut Debugger) -> Result<()> {
        self.saved.replace(dbg.set_breakpoint(self.addr)?);
        self.enabled = true;
        Ok(())
    }

    /// Disabled breakpoitn on debugger
    fn disable(&mut self, dbg: &mut Debugger) -> Result<()> {
        if let Some(byte) = self.saved {
            dbg.remove_breakpoint(self.addr, byte)?;
        }
        self.enabled = false;
        Ok(())
    }
}
