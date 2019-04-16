use std::ffi::CString;
use std::path::Path;
// use std::collections::VecDeque;
// use std::mem::transmute;

use failure::ResultExt;
use nix::sys::ptrace;
use nix::sys::wait::waitpid;
use nix::unistd::{execvp, fork, ForkResult, Pid};

// use capstone::prelude::*;
// use goblin::elf::Elf;

use crate::cli::{Cmd, Fmt};
use crate::error::{ErrorKind, Result};

pub struct Debugger {
    prog: CString,
    args: Vec<CString>,
    // elf: Box<Elf<'a>>,
    // cs: Box<Capstone<'a>>,
    debugged: Option<Box<dyn Debugged>>,
}

pub trait Debugged {
    /// Start debugged program
    fn run(&mut self, args: Vec<CString>);
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
    fn new(prog: CString) -> Box<dyn Debugged> {
        Box::new(Ptraced { prog, pid: None })
    }
}

impl Debugged for Ptraced {
    fn run(&mut self, args: Vec<CString>) {
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

            // Wait for tracee to finish
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
        let prog = CString::new(
            path.as_ref()
                .canonicalize()
                .with_context(|_| ErrorKind::path(&path))?
                .to_str()
                .unwrap(),
        )
        .unwrap();
        let args = args
            .iter()
            .map(|s| CString::new(s.clone()).unwrap())
            .collect::<Vec<_>>();

        // let cs = Capstone::new()
        //     .x86()
        //     .mode(arch::x86::ArchMode::Mode64)
        //     .syntax(arch::x86::ArchSyntax::Intel)
        //     .detail(true)
        //     .build()
        //     .expect("capstone failed");

        // let buf = std::fs::read(std::env::args().nth(1).unwrap())
        //     .expect("read failed");
        // let elf = Elf::parse(&buf).expect("goblin failed");

        Ok(Debugger {
            prog,
            args,
            debugged: None,
            // elf: Box::new(elf),
            // cs: Box::new(cs),
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
            // prompt
        }

        let debugged = Some(Ptraced::new(self.prog.clone()));

        if let Some(target) = self.debugged.as_mut() {
            let args = if args.len() > 0 {
                args.iter()
                    .map(|s| CString::new(s.clone()).unwrap())
                    .collect::<Vec<_>>()
            } else {
                self.args.clone()
            };
            target.run(args);
        }
    }

    fn x_command(&self, fmt: Option<Fmt>, address: Option<u64>) {
        dbg!((fmt, address));
    }

    fn repeat_command(&self) {
        unimplemented!()
    }
}

// match fork().expect("fork failed") {
//     ForkResult::Child => {
//         // Initiate a trace with ptrace(PTRACE_TRACEME, ...)
//         ptrace::traceme().expect("ptrace failed");

//         // Execute PROG with [ARGS]
//         execvp(&prog, &args).expect("execvp failed");
//     }
//     ForkResult::Parent { child } => {
//         // Wait for PTRACE_TRACEME in child
//         let status = waitpid(child, None).expect("waitpid failed");
//         dbg!(status);

//         // Terminate tracee if the tracer exits
//         ptrace::setoptions(child, ptrace::Options::PTRACE_O_EXITKILL)
//             .expect("ptrace failed");

//         // Stop the tracee on the next clone(2)
//         ptrace::setoptions(child, ptrace::Options::PTRACE_O_TRACECLONE)
//             .expect("ptrace failed");

//         // Wait for clone(2) event, tracee should stop execution at _start,
//         // which is likely actually _start inside ld.so(8) for dynamically
//         // linked executables.
//         let event = ptrace::getevent(child).expect("ptrace failed");
//         dbg!(event);

//         // Print memory map
//         std::process::Command::new("/bin/cat")
//             .arg(format!("/proc/{}/maps", child))
//             .status()
//             .expect("command failed");

//         // Read registers from tracee
//         let regs = ptrace::getregs(child).unwrap();
//         println!("rip: 0x{:016x}", regs.rip);

//         let addr = unsafe { transmute(regs.rip) };
//         let word = ptrace::read(child, addr).unwrap();

//         // let entry = unsafe { transmute(vaddr + elf.entry) };
//         // dbg!(entry);

//         // Read basic block from instruction pointer
//         let mut bytes: VecDeque<u8> = VecDeque::new();
//         let mut iaddr = regs.rip; // instruction address
//         let mut raddr = regs.rip; // read address
//         'outer: loop {
//             // Pre-fetch and buffer word from child process
//             let ptr = unsafe { transmute(raddr) };

//             let word = ptrace::read(child, ptr).unwrap().to_le_bytes();
//             for byte in word.iter() {
//                 bytes.push_back(*byte);
//             }

//             // Increment read address
//             raddr += word.len() as u64;

//             // Buffer underrun, minimum of 2 words for disassembly
//             if bytes.len() < (word.len() * 2) {
//                 continue;
//             }

//             // Dequeue 2 words for disassembly
//             let code = bytes.drain(..(word.len() * 2)).collect::<Vec<u8>>();

//             // Disassemble instructions
//             let insns = cs
//                 .disasm_count(&code, iaddr, 1)
//                 .expect("capstone failed (disasm_count)");

//             let insn = insns.iter().nth(0).unwrap();
//             let size = insn.bytes().len();
//             let detail = cs.insn_detail(&insn).unwrap();
//             let groups = detail.groups().collect::<Vec<_>>();

//             println!(
//                 "0x{:016x}:\t{}\t{}",
//                 iaddr,
//                 insn.mnemonic().unwrap(),
//                 insn.op_str().unwrap()
//             );

//             // Push residual bytes to front of queue
//             for byte in code.iter().skip(size).rev() {
//                 bytes.push_front(*byte);
//             }

//             iaddr += size as u64;

//             let insn_group_ids = [
//                 capstone::InsnGroupType::CS_GRP_JUMP,
//                 capstone::InsnGroupType::CS_GRP_CALL,
//                 capstone::InsnGroupType::CS_GRP_RET,
//                 capstone::InsnGroupType::CS_GRP_INT,
//                 capstone::InsnGroupType::CS_GRP_IRET,
//             ];

//             for insn_group_id in &insn_group_ids {
//                 let insn_group =
//                     capstone::InsnGroupId(*insn_group_id as u8);
//                 if groups.contains(&insn_group) {
//                     break 'outer;
//                 }
//             }

//             if iaddr > regs.rip + 20 {
//                 break;
//             }
//         }

//         // let saved = ptrace::read(child, entry).unwrap();
//         // dbg!(saved.to_le_bytes());

//         // Continue
//         ptrace::cont(child, None).unwrap();

//         // Wait for tracee to finish
//         let status = waitpid(child, None).unwrap();
//         dbg!(status);
//     }
// }
