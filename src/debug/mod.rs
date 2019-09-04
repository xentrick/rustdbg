/* Process Debugging Library */

use std::ffi::CString;
use std::path::Path;
use nix::unistd::*;
use nix::sys::ptrace;
use nix::sys::wait::*;
use nix::sys::signal;
use nix::Error;
use nix::errno::Errno;
use inferior::*;
use libc::c_long;

pub mod breakpoint;

pub struct Debuggee {
    file: str,
}

pub fn peek(pid: Pid, addr: InferiorPointer) -> i64 {
    ptrace::read(pid, addr.as_voidptr())
        .ok()
        .expect("Unable to read from address")
}

pub fn continue_exec(proc: &mut Inferior) -> i32 {
    /* Continue with no signal */
    println!("Continuing execution of inferior.");
    ptrace::cont(proc.pid, None)
        .ok()
        .expect("Failed to continue inferior execution.");
    loop {
        proc.state = match waitpid(proc.pid, None) {
            Ok(WaitStatus::Exited(_pid, code)) => return code,
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
}

// Better named as load()?
pub fn start(file: &Path, args: &[&str]) -> Result<Inferior, Error> {
    println!("Executing Debuggee: {}", file.display());

    let mut session = Inferior{ pid: Pid::this(), state: InferiorState::Stopped };

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
    match waitpid(child, None) {
        Ok(WaitStatus::Stopped(child, signal::SIGTRAP)) => {
            println!("Process STOPPED on first instruction.");
            return Ok(Inferior { pid: child, state: InferiorState::Running })
        }
        Ok(WaitStatus::PtraceEvent(child, signal::SIGTRAP, 0)) => {
            println!("SIGTRAP ENCOUNTERED");
            return Ok(Inferior { pid: child, state: InferiorState::Running })
        }
        Ok(_) => panic!("Unexpected stop in attach_inferior"),
        Err(e) => panic!("Error: {}", e)
    }

}

pub fn trace_child(file: &Path, args: &[&str]) -> () {
    println!("Fork resulted in child. Running execve()");
    // Convert `Path` to `CString`
    let cfile = &CString::new(
        file.to_str()
            .unwrap()
    ).expect("Failed to convert file path to CString");

    /* We need to add arg support */
    //let cargs: &[CString] = args.map(|&a| a as CString);
    // let mut cargs: &[&CString] = &[];
    // for a in args {
    //     cargs.append(CString::new(a.as_bytes()));
    // }

    // Begin Tracing parent
    ptrace::traceme()
        .ok()
        .expect("Unable to set PTRACE_TRACEME");

    // Execute with arguments
    execve(cfile, &[], &[]).expect("Failed to run execve()");
    // match execve(cfile, &[], &[]) {
    //     Ok(t) => println!("Exec okay"),
    //     Err(e) => println!("Error: {}", e)
    // }
    unreachable!()
}
