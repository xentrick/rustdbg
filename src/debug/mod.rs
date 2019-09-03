/* Process Debugging Library */

use std::ffi::CString;
use std::path::Path;
use nix::unistd::*;
use nix::sys::ptrace;
use nix::sys::wait::*;
use nix::sys::signal;
use nix::Error;
use inferior::*;


pub struct Debuggee {
    file: str,
}


// Better named as load()
pub fn start(file: &Path, args: &[&str]) {
    println!("Executing Debuggee: {}", file.display());

    // Fork and Verify Result
    match fork() {
        Ok(ForkResult::Child) => trace_child(file, args),
        Ok(ForkResult::Parent { child }) => {
            println!("Fork result was a parent");
            attach(child);
        }
        Err(_) => println!("Fork failed"),
    }

}

pub fn attach(child: Pid) {
    println!("Attaching to PID");
    match waitpid(child, None) {
        Ok(WaitStatus::Stopped(child, signal::SIGTRAP)) => {
            println!("IT WORKED!");
        }
        Ok(WaitStatus::PtraceEvent(child, signal::SIGTRAP, 0)) => println!("IT WORKED!"),
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
    //execve(cfile, &[], &[]).expect("Failed to run execve()");
    execve(cfile, &[], &[]).expect("Failed to run execve()");
}
