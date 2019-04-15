use std::collections::VecDeque;
use std::ffi::CString;
use std::fmt;
// use std::mem::transmute;
use std::path::PathBuf;

use capstone::prelude::*;
// use goblin::elf::Elf;
// use nix::sys::ptrace;
// use nix::sys::wait::waitpid;
// use nix::unistd::{execvp, fork, ForkResult};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use structopt::StructOpt;

mod cli;
use cli::{Cmd, Opt};

struct Debugger<'a> {
    path: PathBuf,
    args: Vec<String>,

    // elf: Box<Elf<'a>>,
    cs: Box<Capstone<'a>>,
}

impl<'a> Debugger<'a> {
    fn new(path: PathBuf, args: Vec<String>) -> Self {
        let cs = Capstone::new()
            .x86()
            .mode(arch::x86::ArchMode::Mode64)
            .syntax(arch::x86::ArchSyntax::Intel)
            .detail(true)
            .build()
            .expect("capstone failed");

        // let buf = std::fs::read(std::env::args().nth(1).unwrap())
        //     .expect("read failed");
        // let elf = Elf::parse(&buf).expect("goblin failed");

        return Debugger {
            path,
            args,
            // elf: Box::new(elf),
            cs: Box::new(cs),
        };
    }

    fn run_command(&self, cmd: Cmd) {
        dbg!(cmd);
        //
    }

    fn repeat_command(&self) {
        unimplemented!()
    }
}

fn parse_command(line: String) -> Result<Cmd, structopt::clap::Error> {
    let line = match &line[..2] {
        "x/" | "p/" => line.replacen("/", " ", 1),
        _ => line,
    };
    Cmd::from_iter_safe(line.split_whitespace())
}

fn main() -> Result<(), Box<std::error::Error>> {
    let opt = Opt::from_args();
    let debugger = Debugger::new(opt.prog, opt.args);

    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline("(dbg) ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_ref());
                match parse_command(line) {
                    Ok(cmd) => debugger.run_command(cmd),
                    Err(_e) => println!("{}", _e),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Quit");
            }
            Err(ReadlineError::Eof) => {
                println!("quit");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    // let prog = CString::new(opt.prog.canonicalize()?.to_str().unwrap())?;
    // let args = opt
    //     .args
    //     .iter()
    //     .map(|s| CString::new(s.clone()).unwrap())
    //     .collect::<Vec<_>>();

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
    Ok(())
}
