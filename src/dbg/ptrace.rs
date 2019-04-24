use std::ffi::CString;
use std::path::Path;

use nix::sys::{
    ptrace,
    wait::{waitpid, WaitStatus},
};
use nix::unistd::{execvp, fork, ForkResult, Pid};

use super::{Address, Debugged, ErrorKind, Result};

/// Debugging interface for platforms that support ptrace (2)
#[derive(Debug)]
pub struct Ptraced {
    prog: CString,
    pid: Option<Pid>,
    status: Option<WaitStatus>,
}

impl Ptraced {
    pub fn new<P: AsRef<Path>>(path: P) -> Box<dyn Debugged> {
        let prog = CString::new(path.as_ref().to_str().unwrap())
            .expect("null byte in string");
        Box::new(Ptraced {
            prog,
            pid: None,
            status: None,
        })
    }

    fn pid(&self) -> Result<Pid> {
        Ok(self.pid.ok_or(ErrorKind::NotRunning)?)
    }

    fn read_word(&mut self, addr: Address) -> Result<[u8; 8]> {
        Ok(ptrace::read(self.pid()?, addr as ptrace::AddressType)
            .map(|word| word.to_le_bytes())
            .map_err(|_| ErrorKind::Read(addr))?)
    }

    fn write_word(&mut self, addr: Address, word: [u8; 8]) -> Result<()> {
        let word = Address::from_le_bytes(word) as ptrace::AddressType;
        Ok(
            ptrace::write(self.pid()?, addr as ptrace::AddressType, word)
                .map_err(|_| ErrorKind::Write(addr))?,
        )
    }
}

impl Drop for Ptraced {
    fn drop(&mut self) {
        if let Some(child) = self.pid {
            if let Err(e) = ptrace::kill(child) {
                eprintln!("error: kill: {} ({})", e, child);
            }
            dbg!(waitpid(child, None).expect("waitpid"));
        }
    }
}

impl Debugged for Ptraced {
    fn run(&mut self, args: Vec<String>) {
        let mut args = args
            .iter()
            .map(|s| CString::new(s.clone()).unwrap())
            .collect::<Vec<_>>();

        args.insert(0, self.prog.to_owned());

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

                // self.cont();
            }
        }
    }

    fn cont(&mut self) -> Result<()> {
        let pid = self.pid()?;

        // Continue
        ptrace::cont(pid, None).unwrap();

        // Wait for debugged program
        let status = waitpid(pid, None);

        match status {
            Ok(WaitStatus::Exited(pid, _code)) => {
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
        Ok(())
    }

    fn read(&mut self, vaddr: Address, size: usize) -> Result<Vec<u8>> {
        // Read data into vector of bytes
        let mut data = Vec::new();
        let mut addr = vaddr;
        while data.len() < size {
            let word = self.read_word(addr)?;
            data.extend_from_slice(&word);
            addr += std::mem::size_of::<ptrace::AddressType>();
        }
        data.truncate(size);

        Ok(data)
    }

    fn write(&mut self, vaddr: Address, data: &[u8]) -> Result<usize> {
        // Write full word-sized values
        let word_size = std::mem::size_of::<Address>();
        let mut iter = data.chunks_exact(word_size);
        let mut i = 0;
        while let Some(slice) = iter.next() {
            // Convert byte slice to a word-sized value
            let mut word = [0; 8];
            word.copy_from_slice(slice);
            self.write_word(vaddr + (i * word_size), word)?;
            i += 1;
        }

        // Write partial word value
        let mut iter = iter.remainder().iter().rev();
        let mut j = 0;
        if iter.len() > 0 {
            let mut word = self.read(vaddr + (i * word_size), word_size)?;
            while let Some(byte) = iter.next() {
                word[j] = *byte;
                j += 1;
            }
            self.write(vaddr + (i * word_size), &word)?;
        }

        Ok((i + j) * word_size)
    }

    fn step(&mut self, count: usize) {
        dbg!(count);
        unimplemented!()
    }
}
