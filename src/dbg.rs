use std::ffi::CString;
use std::path::{Path, PathBuf};

use failure::ResultExt;
use nix::sys::ptrace;
use nix::sys::wait::waitpid;
use nix::unistd::{execvp, fork, ForkResult, Pid};

use crate::cli::{prompt_yes_no, Cmd, Fmt};
use crate::error::{ErrorKind, Result};

pub struct Debugger {
    prog: PathBuf,
    args: Vec<String>,
    debugged: Option<Box<dyn Debugged>>,
}

pub trait Debugged {
    /// Start debugged program
    fn run(&mut self, args: Vec<String>);
    /// Continue program execution
    fn cont(&mut self);
    /// Step one instruction exactly
    fn stepi(&mut self, count: usize);
    /// Set breakpoint at specified location
    fn breakpoint(&mut self, vaddr: u64);
    /// Read memory of debugged program
    fn read(&mut self, vaddr: u64, size: usize);
}

struct Ptraced {
    prog: CString,
    pid: Option<Pid>,
}

impl Ptraced {
    fn new<P: AsRef<Path>>(path: P) -> Box<dyn Debugged> {
        let prog = CString::new(path.as_ref().to_str().unwrap())
            .expect("null byte in string");
        Box::new(Ptraced { prog, pid: None })
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
                let status = waitpid(child, None).expect("waitpid failed");
                dbg!(status);

                // Terminate tracee if the tracer exits
                ptrace::setoptions(child, ptrace::Options::PTRACE_O_EXITKILL)
                    .expect("ptrace failed");

                // Stop the tracee on the next clone(2)
                ptrace::setoptions(child, ptrace::Options::PTRACE_O_TRACECLONE)
                    .expect("ptrace failed");

                // Wait for clone(2) event, tracee should stop execution at
                // _start, which is likely actually _start inside ld.so(8) for
                // dynamically linked executables.
                let event = ptrace::getevent(child).expect("ptrace failed");
                dbg!(event);

                // ptrace::cont(child, None).unwrap();
            }
        }
    }

    fn cont(&mut self) {
        if let Some(pid) = self.pid {
            ptrace::cont(pid, None).unwrap();

            // Wait for debugged program
            let status = waitpid(pid, None).unwrap();
            dbg!(status);
        }
    }

    fn stepi(&mut self, count: usize) {
        unimplemented!()
    }

    fn breakpoint(&mut self, vaddr: u64) {
        unimplemented!()
    }

    fn read(&mut self, vaddr: u64, size: usize) {
        unimplemented!()
    }
}

/// Interactive debugger type
impl Debugger {
    pub fn new<P: AsRef<Path>>(path: P, args: Vec<String>) -> Result<Self> {
        let prog = path
            .as_ref()
            .canonicalize()
            .with_context(|_| ErrorKind::path(&path))?;

        Ok(Debugger {
            prog,
            args,
            debugged: None,
        })
    }

    pub fn exec(&mut self, cmd: Cmd) {
        match cmd {
            Cmd::Break { loc } => self.break_command(loc),
            Cmd::Examine { fmt, address } => self.x_command(fmt, address),
            Cmd::Run { args } => self.run_command(args),
            Cmd::Repeat => self.repeat_command(),
        }
    }

    fn break_command(&self, loc: u64) {
        unimplemented!()
    }

    fn run_command(&mut self, args: Vec<String>) {
        if self.debugged.is_some() {
            println!("The program being debugged has been started already.");
            if !prompt_yes_no("Start it from the beginning?") {
                println!("Program not restarted.");
                return;
            }
        }
        println!(
            "Starting program: {} {}",
            self.prog.display(),
            self.args.join(" "),
        );

        self.debugged = Some(Ptraced::new(self.prog.clone()));
        if let Some(target) = self.debugged.as_mut() {
            if args.len() > 0 {
                self.args = args;
            }
            target.run(self.args.clone());
        }
    }

    fn x_command(&self, fmt: Option<Fmt>, address: Option<u64>) {
        dbg!((fmt, address));
    }

    fn repeat_command(&self) {
        unimplemented!()
    }
}
