use std::ffi::CString;
use std::path::{Path, PathBuf};

use failure::ResultExt;
use nix::sys::{
    ptrace,
    wait::{waitpid, WaitStatus},
};
use nix::unistd::{execvp, fork, ForkResult, Pid};

mod error;
pub use error::{Error, ErrorKind, Result};

/// Debugger with generic debugged progam type
pub struct Debugger {
    prog: PathBuf,
    debugged: Option<Box<dyn Debugged>>,
}

/// Generic debugged program type
trait Debugged {
    /// Set breakpoint at specified location
    fn breakpoint(&mut self, vaddr: u64);
    /// Continue program execution
    fn cont(&mut self);
    /// Read memory of debugged program
    fn read(&mut self, vaddr: u64, size: usize);
    /// Start debugged program
    fn run(&mut self, args: Vec<String>);
    /// Step one instruction exactly
    fn step(&mut self, count: usize);
}

/// Debugging interface for platforms that support ptrace (2)
struct Ptraced {
    prog: CString,
    pid: Option<Pid>,
    status: Option<WaitStatus>,
}

impl Ptraced {
    fn new<P: AsRef<Path>>(path: P) -> Box<dyn Debugged> {
        let prog = CString::new(path.as_ref().to_str().unwrap())
            .expect("null byte in string");
        Box::new(Ptraced {
            prog,
            pid: None,
            status: None,
        })
    }
}

impl Drop for Ptraced {
    fn drop(&mut self) {
        if let Some(child) = self.pid {
            if let Err(e) = ptrace::kill(child) {
                eprintln!("error: kill: {} ({})", e, child);
            }
            dbg!(waitpid(child, None));
        }
    }
}

impl Debugged for Ptraced {
    fn run(&mut self, args: Vec<String>) {
        let args = args
            .iter()
            .map(|s| CString::new(s.clone()).unwrap())
            .collect::<Vec<_>>();

        // TODO: replace expects
        match fork().expect("fork failed") {
            ForkResult::Child => {
                // Initiate a trace with ptrace(PTRACE_TRACEME, ...)
                ptrace::traceme().expect("ptrace failed");

                // Execute PROG with [ARGS]
                execvp(&self.prog, &args).expect("execvp failed");
            }
            ForkResult::Parent { child } => {
                self.pid = Some(child);

                // Wait for PTRACE_TRACEME in child
                let _ = waitpid(child, None).expect("waitpid failed");

                // Terminate tracee if the tracer exits
                ptrace::setoptions(child, ptrace::Options::PTRACE_O_EXITKILL)
                    .expect("ptrace failed");

                // Stop the tracee on the next clone(2)
                ptrace::setoptions(child, ptrace::Options::PTRACE_O_TRACECLONE)
                    .expect("ptrace failed");

                // Wait for clone(2) event, tracee should stop execution at
                // _start, which is likely actually _start inside ld.so(8) for
                // dynamically linked executables.
                let _ = ptrace::getevent(child).expect("ptrace failed");

                self.cont();
            }
        }
    }

    fn cont(&mut self) {
        if let Some(pid) = self.pid {
            ptrace::cont(pid, None).unwrap();

            // Wait for debugged program
            let status = waitpid(pid, None);

            match status {
                Ok(WaitStatus::Exited(pid, code)) => {
                    println!("[Inferior 1 (process {}) exited normally]", pid);
                    self.pid = None;
                }
                Ok(WaitStatus::Signaled(_, signal, _)) => {
                    println!("Program received signal {}", signal);
                }
                Err(e) => {
                    println!("error: waitpid: {}", e);
                }
                Ok(_) => (),
            }

            self.status = status.ok();
        } else {
            println!("The program is not being run.");
        }
    }

    fn breakpoint(&mut self, vaddr: u64) {
        unimplemented!()
    }

    fn read(&mut self, vaddr: u64, size: usize) {
        unimplemented!()
    }

    fn step(&mut self, count: usize) {
        unimplemented!()
    }
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

    pub fn breakpoint(&self, loc: u64) {
        unimplemented!()
    }

    pub fn cont(&mut self) {
        if let Some(target) = self.debugged.as_mut() {
            target.cont()
        }
    }

    pub fn run(&mut self, args: Vec<String>) {
        // if self.debugged.is_some() {
        // }

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

    // pub fn examine(
    //     &self,
    //     addr: Option<u64>,
    //     reverse: bool,
    //     repeat: usize,
    //     size: char,
    //     format: char,
    // ) {
    //     unimplemented!()
    // }

    pub fn repeat(&self) {
        unimplemented!()
    }
}
