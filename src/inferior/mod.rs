/* Inferior Debugging
* Credit has to go to Brandon Falk for the start of this code.
* I used some of his code to get started in making a well structured debugger.
* Code was then ported to nix with changes where I felt necessary.
*/

use nix::errno::Errno;
use nix::Error;
use nix::sys::{ptrace, signal};
use nix::sys::wait::*;
use nix::unistd::{
    execve,
    fork,
    ForkResult,
    Pid
};

//use libc::c_void;

//use std::cell::RefCell;
use std::collections::{HashSet, HashMap};
use std::default::Default;
use std::ffi::CString;
//use std::fmt;
//use std::path::Path;
//use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

pub mod ffi;

/// Tracks if an exit has been requested via the Ctrl+C/Ctrl+Break handler
static EXIT_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Function invoked on module loads
/// (debugger, module filename, module base)
//type ModloadFunc = Box<dyn Fn(&mut Inferior, &str, usize)>;

/// Function invoked on debug events
//type DebugEventFunc = Box<dyn Fn(&mut Inferior, &DEBUG_EVENT)>;

/// Function invoked on breakpoints
/// (debugger, tid, address of breakpoint,
///  number of times breakpoint has been hit)
/// If this returns false the debuggee is terminated
type BreakpointCallback = fn(&mut Inferior, u32, usize, u64) -> bool;

/// Ctrl+C handler so we can remove breakpoints and detach from the debugger
unsafe extern "system" fn ctrl_c_handler(_ctrl_type: u32) -> i32 {
    // Store that an exit was requested
    EXIT_REQUESTED.store(true, Ordering::SeqCst);

    // Sleep forever
    loop {
        std::thread::sleep(Duration::from_secs(100));
    }
}

/// Different types of breakpoints
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BreakpointType {
    Freq,  // Keep the breakpoint and track it.
    Single,  // Delete BP after hit
}

#[derive(Clone)]
pub enum InferiorState {
    Startup,
    Running,
    Stopped,
    Sleeping,
    Zombie,
    Dead,
    Raised,  /* Traps (Ex: Single step) */
}

/// Structure to represent breakpoints
#[derive(Clone)]
pub struct Breakpoint {
    /// Offset from module base
    offset: usize,

    /// Tracks if this breakpoint is currently active
    enabled: bool,

    /// Original byte that was at this location, only set if breakpoint was
    /// ever applied
    orig_byte: Option<u8>,

    /// Tracks if this breakpoint should stick around after it's hit once
    typ: BreakpointType,

    /// Name of the function this breakpoint is in
    funcname: Arc<String>,

    /// Offset into the function that this breakpoint addresses
    funcoff:  usize,

    /// Module name
    modname: Arc<String>,

    /// Callback to invoke if this breakpoint is hit
    callback: Option<BreakpointCallback>,

    /// Number of times this breakpoint has been hit
    freq: u64,
}


#[derive(Clone)]
pub struct Inferior {
    /* Process Information */
    pub pid: Pid,
    pub tids: HashMap<u32, Pid>,  // Threads
    pub attached: bool,

    /* Startup data */
    pub location: String,
    pub args: HashMap<u32, String>,
    pub env: HashMap<String, String>,

    /* Process State */
    pub state: InferiorState,
    pub aslr: bool,
    //pub mem: MemoryMap,

    /* Breakpoints */
    breakpoints: HashMap<usize, Breakpoint>,
    breakpoint_bounds: HashMap<String, (usize, usize)>,  // Track minimum and maximum addresses for breakpoints per module

    /* Callbacks */
    //module_load_callbacks: Option<Vec<ModloadFunc>>,  // Invoked when a module is loaded
    // debug_event_callbacks: Option<Vec<DebugEventFunc>>,

    /// List of all PCs we hit during execution
    /// Keyed by PC
    /// Tuple is (module, offset, symbol+offset, frequency)
    /// IM NOT REALLY SURE WAHT THIS IS
    // coverage: HashMap<usize, (Arc<String>, usize, String, u64)>, 

    /* Shared Libraries */
    modules: HashSet<(String, usize)>,

    /* TIDs actively single stepping mapped to the PC they stepped from */
    single_step: HashMap<u32, usize>,

    /* Frequent Tracking, disable print to screen */
    /* Disabled by default */
    always_freq: bool,

    /* Last time the database was saved */
    last_db_save: Instant,

    /* Prints more status informatin during runtime */
    verbose: bool,

    /* Time we attached to the target */
    start_time: Instant,

    /* Kill requested */
    kill_requested: bool,

    /* Session Info */
    //steps: u32,
    //reason: Reason,
    //recoil_mode: RecoilMode,
    //stopaddr: u64,
}

/* Default Impelementation for Inferior */
impl<'a> Default for Inferior {
    fn default() -> Inferior {
        Inferior {
            pid: Pid::this(),
            tids: HashMap::new(),
            attached: false,

            location: String::new(),
            args: HashMap::new(),
            env: HashMap::new(),

            state: InferiorState::Startup,
            aslr: true,  // Get System Information for this...

            breakpoints: HashMap::new(),
            breakpoint_bounds: HashMap::new(),

            modules: HashSet::new(),
            //module_load_callbacks: Some(Vec::new()),
            //debug_event_callbacks: Some(Vec::new()),

            single_step: HashMap::new(),
            always_freq: false,

            last_db_save: Instant::now(),
            verbose: false,

            start_time: Instant::now(),
            kill_requested: false,
        }
    }
}

/* Inferior Implementation */
impl<'a> Inferior {

    /* Start new process */
    pub fn start(file: String, args: &[String]) -> Inferior {
        println!("Executing: {}", file);

        let inf: Inferior;

        match fork() {
            Ok(ForkResult::Child) => return Inferior::attach_self(file, args),
            Ok(ForkResult::Parent { child }) => {
                inf.pid = child;
                return Inferior::wait()
            }
            Err(e) => panic!("Fork failed: {}", e),
        }
    }

    /* Attach to a PID */
    pub fn attach(pid: u32) -> Inferior {
        println!("Implement raw attach");
        Inferior::default()
    }

    pub fn attach_self(file: String, args: &[String]) -> Inferior {
        println!("attach_self");

        let mut inf = Inferior::default();
        inf.location = file;

        let cmd = CString::new(inf.location).unwrap();

        // *** Implement args and environment ***
        //let args: = &[];
        //let env = &[];

        /* We need to add arg support */
        //let cargs: &[CString] = args.map(|&a| a as CString);
        // let mut cargs: &[&CString] = &[];
        // for a in args {
        //     cargs.append(CString::new(a.as_bytes()));
        // }

        // *** For now don't deal with ASLR (CHANGE LATER) ***
        ffi::disable_aslr();
        inf.aslr = false;

        // Begin Tracing
        println!("Setting traceme()");
        ptrace::traceme()
            .ok()
            .expect("Unable to set PTRACE_TRACEME");

        // Execute with arguments
        println!("execve(\"{}\")", cmd.clone().into_string().unwrap());
        execve(&cmd, &[], &[]).expect("Failed to run execve()");
        unreachable!()

    }

    pub fn wait(&mut self) -> Inferior {
        /* Call waitpid to get a status */
        match waitpid(self.pid, None) {
            Ok(WaitStatus::Stopped(pid, signal::SIGTRAP)) => {
                println!("Process STOP encountered.");
                return Inferior { pid: pid, state: InferiorState::Running, attached: true, ..Inferior::default() }
            }
            Ok(WaitStatus::PtraceEvent(pid, signal::SIGTRAP, 0)) => {
                println!("SIGTRAP encountered.");
                return Inferior { pid: pid, state: InferiorState::Running, attached: true, ..Inferior::default() }
            }
            Ok(_) => panic!("Unhandled event in waitpid. Implement feature."),
            Err(e) => panic!("Error: {}", e)
        }
    }

    pub fn r#continue(&mut self) {
        println!("Continuing execution...");
        ptrace::cont(self.pid, None)
            .ok()
            .expect("Failed to continue process execution.");
        Inferior::wait()
    }

}


/* Helpers */

/* Get elapsed time in seconds */
fn elapsed_from(start: &Instant) -> f64 {
    let dur = start.elapsed();
    dur.as_secs() as f64 + dur.subsec_nanos() as f64 / 1_000_000_000.0
}
