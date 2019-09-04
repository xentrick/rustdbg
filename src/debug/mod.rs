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


pub struct Debuggee {
    file: str,
}

// Better named as load()?
pub fn start(file: &Path, args: &[&str]) {
    println!("Executing Debuggee: {}", file.display());

    // Fork and Verify Result
        match fork() {
            Ok(ForkResult::Child) => trace_child(file, args),
            Ok(ForkResult::Parent { child }) => {
                attach(child);
            }
            Err(Error::Sys(Errno::EAGAIN)) => println!("Sys AGAIN error"),
            Err(e) => {
                println!("Fork failed: {}", e);
                return ()
            }
        }

}

pub fn attach(child: Pid) -> Result<Inferior, Error> {
    println!("Fork result was a parent");
    println!("Attaching to PID: {}", child);
    match waitpid(child, None) {
        Ok(WaitStatus::Stopped(child, signal::SIGTRAP)) => {
            println!("Process STOPPED on first instruction.");
            return Ok(Inferior { pid: child, state: InferiorState::Running })
        }
        Ok(WaitStatus::PtraceEvent(child, signal::SIGTRAP, 0)) => println!("SIGTRAP ENCOUNTERED"),
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
    ptrace::traceme().expect("Unable to set PTRACE_TRACEME");
    // Execute with arguments
    execve(cfile, &[], &[]).expect("Failed to run execve()");
    // match execve(cfile, &[], &[]) {
    //     Ok(t) => println!("Exec okay"),
    //     Err(e) => println!("Error: {}", e)
    // }
    unreachable!()
}
