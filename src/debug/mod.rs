/* Process Debugging Library */

use std::ffi::CString;
use std::path::Path;
use nix::unistd::*;
use nix::sys::ptrace;
use nix::sys::wait::*;
use nix::sys::signal;
use nix::Error;
use nix::errno::Errno;
use libc::{c_long, c_void};

pub mod breakpoint;
use inferior::*;

mod ffi {
    use libc::{c_int, c_long};

    extern {
        pub fn personality(persona: c_long) -> c_int;
    }
}

fn disable_aslr() -> () {
    unsafe {
        let old = ffi::personality(0xffffffff);
        ffi::personality((old | 0x0040000) as i64);
    }
}

pub struct Debuggee {
    file: str,
}

pub fn write(pid: Pid, addr: InferiorPointer, data: i64) -> () {
    ptrace::write(pid, addr.as_voidptr(), data as * mut c_void)
        .ok()
        .expect("Failed to write data to inferior");
}

pub fn peek(pid: Pid, addr: InferiorPointer) -> Result<i64, Error> {
    println!("[{}] Peeking WORD at {:#x}", pid, addr);
    ptrace::read(pid, addr.as_voidptr())
}

pub fn continue_exec(mut proc: Inferior) -> i32 {
    /* Continue with no signal */
    println!("Continuing execution of inferior.");
    ptrace::cont(proc.pid, None)
        .ok()
        .expect("Failed to continue inferior execution.");
    loop {
        proc.state = match waitpid(proc.pid, None) {
            Ok(WaitStatus::Exited(_pid, code)) => {
                println!("Process exited: {}", code);
                return code
            },
            Ok(WaitStatus::Stopped(_pid, signal::SIGTRAP)) => {
                println!("Implement breakpoint handling.");
                InferiorState::Stopped
            },
            Ok(WaitStatus::Stopped(_pid, signal)) => {
                panic!("Unexpected stop on signal {} when continuing execution. State: {}", signal, proc.state as i32)
            }
            Ok(_) => panic!("Unspexted stop in continue."),
            Err(_) => panic!("Unhandled error in continue.")
        };
    }
    println!("Inferior State: {}", proc.state as i32);
    println!("Attached: {}", proc.attached);
}

// Better named as load()?
pub fn start(file: &Path, args: &[&str]) -> Result<Inferior, Error> {
    println!("Executing Debuggee: {}", file.display());

    let mut session = Inferior::default();

    // Fork and Verify Result
    match fork() {
        Ok(ForkResult::Child) => trace_child(file, args),
        Ok(ForkResult::Parent { child }) => {
            // Implement error handling in Inferior struct. Use `?`
            session = attach(child).expect("Got inferior!");
        }
        Err(Error::Sys(Errno::EAGAIN)) => println!("Sys AGAIN error"),
        Err(e) => {
            println!("Fork failed: {}", e);
            return Err(e)
        }
    }

    Ok(session)
}

pub fn attach(child: Pid) -> Result<Inferior, Error> {
    println!("Fork result was a parent");
    println!("Attaching to PID: {}", child);
    /*
    * GDB adds a main thread. If target extends ptrace
    * target, it should decorate `ptid` later. inf-ptrace.c
    */
    match waitpid(child, None) {
        Ok(WaitStatus::Stopped(child, signal::SIGTRAP)) => {
            println!("Process STOPPED on first instruction.");
            //loop {}
            return Ok(Inferior {
                pid: child,
                state: InferiorState::Running,
                attached: true,
                ..Inferior::default()
            })
        }
        Ok(WaitStatus::PtraceEvent(child, signal::SIGTRAP, 0)) => {
            println!("SIGTRAP ENCOUNTERED");
            return Ok(Inferior {
                pid: child,
                state: InferiorState::Running,
                attached: true,
                ..Inferior::default()
            })
        }
        Ok(_) => panic!("Unexpected stop in attach_inferior"),
        Err(e) => panic!("Error: {}", e)
    }

}

pub fn trace_child(file: &Path, args: &[&str]) -> () {
    println!("Fork resulted in child. Running execve()");
    // Convert `Path` to `CString`
    let cfile = &CString::new(file.to_str().unwrap())
        .expect("Failed to convert file path to CString");

    /* We need to add arg support */
    //let cargs: &[CString] = args.map(|&a| a as CString);
    // let mut cargs: &[&CString] = &[];
    // for a in args {
    //     cargs.append(CString::new(a.as_bytes()));
    // }

    // For now don't deal with ASLR (CHANGE LATER)
    disable_aslr();

    // Begin Tracing
    println!("Running traceme()");
    ptrace::traceme()
        .ok()
        .expect("Unable to set PTRACE_TRACEME");

    // Execute with arguments
    println!("execve(\"{}\")", file.display());
    execve(cfile, &[], &[]).expect("Failed to run execve()");
    // match execve(cfile, &[], &[]) {
    //     Ok(t) => println!("Exec okay"),
    //     Err(e) => println!("Error: {}", e)
    // }
    unreachable!()
}
