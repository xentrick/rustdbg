/* Inferior Debugging Structure
* Thanks to Brandon Falk as I used one of his windows debuggers
* as a base outline for writing my own (Sorry). His code has
* taught me quite a bit and deserves recognition.
*/


//use libc::c_void;
use libc::user_regs_struct;

use nix::errno::Errno::*;
use nix::Error;
use nix::Error::Sys;
use nix::sys::{ptrace, signal};
use nix::sys::wait::*;
use nix::ucontext::UContext;
use nix::unistd::{
    execve,
    fork,
    getcwd,
    ForkResult,
    Pid
};

//use std::cell::RefCell;
//use std::boxed::FnBox;
use std::collections::{HashSet, HashMap};
use std::default::Default;
use std::ffi::CString;
//use std::fmt;
//use std::path::Path;
//use std::rc::Rc;
use std::io::stderr;
use std::io::stdout;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use std::unimplemented;

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
    unimplemented!();
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

#[derive(Clone, PartialEq, Eq)]
pub enum InferiorState {
    Startup,
    Running,
    Stopped,
    Sleeping,
    Zombie,
    Dead,
    Raised,  /* Traps (Ex: Single step) */
    None,
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
    pid: Pid,
    tids: HashMap<u32, Pid>,  // Threads
    attached: bool,

    /* Startup data */
    location: String,
    args: HashMap<u32, String>,
    env: HashMap<String, String>,
    cwd: PathBuf,

    /* Process State */
    state: InferiorState,
    aslr: bool,

    //pub mem: MemoryMap,

    /* Breakpoints */
    breakpoints: HashMap<usize, Breakpoint>,
    target_breakpoints: HashMap<String, Vec<Breakpoint>>,
    breakpoint_bounds: HashMap<String, (usize, usize)>,  // Track minimum and maximum addresses for breakpoints per module

    /* Callbacks */
    // module_load_callbacks: Option<Vec<ModloadFunc>>,  // Invoked when a module is loaded
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

    /* Thread Context */
    context: UContext,
    /* Session Info */
    //steps: u32,
    //reason: Reason,
    //recoil_mode: RecoilMode,
    //stopaddr: u64,
}

/* Default Impelementation for Inferior */
impl Default for Inferior {
    fn default() -> Inferior {
        Inferior {
            pid: Pid::this(),
            tids: HashMap::new(),
            attached: false,

            location: String::new(),
            args: HashMap::new(),
            env: HashMap::new(),
            cwd: PathBuf::new(),

            state: InferiorState::None,
            aslr: true,  // Get System Information for this...

            breakpoints: HashMap::new(),
            target_breakpoints: HashMap::new(),
            breakpoint_bounds: HashMap::new(),

            modules: HashSet::new(),
            // module_load_callbacks: Some(Vec::new()),
            // debug_event_callbacks: Some(Vec::new()),

            single_step: HashMap::new(),
            always_freq: false,

            last_db_save: Instant::now(),
            verbose: false,

            start_time: Instant::now(),
            kill_requested: false,

            //context: std::ptr::null_mut(),
            //context: unsafe { std::mem::zeroed() },
            context: UContext::get().unwrap(),
        }
    }
}

/* Inferior Implementation */
impl Inferior {

    // Initialize default Inferior
    pub fn new() -> Self {
        Inferior { ..Inferior::default() }
    }

    /* Start new process */
    pub fn start(&mut self, file: String, args: &[String]) {
        println!("Executing: {}", file);

        self.location = file;
        // self.args: ***FIXME***,
        self.cwd = getcwd().unwrap();
        self.state = InferiorState::Startup;

        // Flush stdio
        stdio_flush();

        match fork() {
            Ok(ForkResult::Child) => self.attach_self(),
            Ok(ForkResult::Parent { child }) => {
                self.pid = child;
                self.attached = true;
                self.prefetch_inferior_data();
                self.wait();
            }
            Err(e) => {
                self.state = InferiorState::Dead;
                println!("Fork failed: {}", e);
            }
        }
    }


    /* Attach to a PID */
    pub fn attach(&mut self, _pid: u32) {
        unimplemented!();
        println!("Attaching to pid {}", _pid);
    }

    pub fn attach_self(&mut self) {
        println!("attach_self");

        let cmd = CString::new(self.location.clone()).unwrap();

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
        self.aslr = false;

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

    pub fn wait(&mut self) {
        /* Call waitpid to get a status */
        loop {

            // Check if it's requested that we exit
            if EXIT_REQUESTED.load(Ordering::SeqCst) {
                // Exit out of the run loop
                break;
            }

            match waitpid(self.pid, None) {
                Ok(WaitStatus::Stopped(pid, signal::SIGTRAP)) => {
                    println!("Process STOP encountered.");
                    self.state = InferiorState::Running;
                    return
                },
                Ok(WaitStatus::PtraceEvent(pid, signal::SIGTRAP, 0)) => {
                    println!("SIGTRAP encountered.");
                    self.state = InferiorState::Running;
                    return
                },
                Ok(WaitStatus::Signaled(pid, sig, core)) => {
                    println!("Signal: {} Pid: {}", sig, pid);
                    if core { println!("Process generated core dump!!!!!!"); }
                    unimplemented!();
                },
                Ok(WaitStatus::PtraceEvent(pid, sig, event)) => {
                    println!("Signal: {} Event: {} Pid: {}", sig, event, pid);
                    unimplemented!();
                },
                Ok(WaitStatus::PtraceSyscall(pid)) => {
                    println!("Process stopped by execution of a system call. `:PTRACDE_O_TRACESYSGOOD` is in effect");
                    unimplemented!();
                },
                Ok(WaitStatus::Continued(pid)) => {
                    println!("Process encountered WaitStatus::Continued.");
                },
                Ok(WaitStatus::Exited(pid, code)) => println!("Process exited. Pid: {} Code: {}", pid, code),
                Ok(WaitStatus::StillAlive) => continue,
                Ok(_) => println!("Unhandled event in waitpid. Implement feature."),
                Err(_) => self.handle_error(),
            }
            if self.state == InferiorState::Dead { break }
        }
    }

    pub fn handle_error(&mut self) {
        let e = Error::last();
        println!("{}", e);
        match e {
            Sys(ECHILD) => self.state = InferiorState::Dead,
            _ => panic!("Unhandled error from kernel."),
        }
    }

    pub fn resume(&mut self) {
        println!("Continuing execution...");
        ptrace::cont(self.pid, None)
            .ok()
            .expect("Failed to continue process execution.");
        self.wait();
    }

    //pub fn set_breakpoint

    // Fix `func`
    pub fn register_modload_callback(&mut self, func: &str) {
        unimplemented!();
    }

    pub fn registers(&mut self) -> user_regs_struct {
        ptrace::getregs(self.pid).expect("Failed to fetch register information.")
    }

    /// Registers a breakpoint for a specific file
    /// `module` is the name of the module we want to apply the breakpoint to,
    /// for example "notepad.exe", `offset` is the byte offset in this module
    /// to apply the breakpoint to
    ///
    /// `name` and `nameoff` are completely user controlled and are used to
    /// give this breakpoint a unique name. Often if used from mesos `name`
    /// will correspond to the function name and `nameoff` will be the offset
    /// into the function. However these can be whatever you like. It's only
    /// for readability of the coverage data
    pub fn register_breakpoint(&mut self, module: Arc<String>, offset: usize,
            name: Arc<String>, nameoff: usize, typ: BreakpointType,
            callback: Option<BreakpointCallback>) {
        // Create a new entry if none exists
        if !self.target_breakpoints.contains_key(&**module) {
            self.target_breakpoints.insert(module.to_string(), Vec::new());
        }

        if !self.breakpoint_bounds.contains_key(&**module) {
            self.breakpoint_bounds.insert(module.to_string(), (!0, 0));
        }

        let mmbp = self.breakpoint_bounds.get_mut(&**module).unwrap();
        mmbp.0 = std::cmp::min(mmbp.0, offset as usize);
        mmbp.1 = std::cmp::max(mmbp.1, offset as usize);

        // Append this breakpoint
        self.target_breakpoints.get_mut(&**module).unwrap().push(
            Breakpoint {
                offset:    offset as usize,
                enabled:   false,
                typ:       typ,
                orig_byte: None,
                funcname:  name.clone(),
                funcoff:   nameoff,
                modname:   module.clone(),
                freq:      0,
                callback,
            }
        );
    }

    pub fn set_always_freq(&mut self, val: bool) { self.always_freq = val; }
    pub fn set_verbose(&mut self, val: bool)     { self.verbose     = val; }
    //pub fn set_bp_print(&mut self, val: bool)    { self.bp_print    = val; }

    fn prefetch_inferior_data(&mut self) {
        // Get TID context data
        self.context = UContext::get().unwrap();
        println!("Context: {:?}", self.context);
    }

    /// Resolves the file name of a given memory mapped file in the target
    /// process
    fn filename_from_module_base(&self, base: usize) -> String {
        unimplemented!();
        // Use GetMappedFileNameW() to get the mapped file name
        let mut buf = [0u16; 4096];
        // let fnlen = unsafe {
        //     GetMappedFileNameW(self.process_handle(),
        //                        base as *mut _, buf.as_mut_ptr(), buf.len() as u32)
        // };
        // assert!(fnlen != 0 && (fnlen as usize) < buf.len(),
        //         "GetMappedFileNameW() failed");

        // // Convert the name to utf-8 and lowercase it
        // let path = String::from_utf16(&buf[..fnlen as usize]).unwrap()
        //     .to_lowercase();

        // // Get the filename from the path
        // Path::new(&path).file_name().unwrap().to_str().unwrap().into()
    }

    /// Add the module loaded at `base` in the target process to our module
    /// list
    fn register_module(&mut self, base: usize) {
        let filename = self.filename_from_module_base(base);

        // Insert into the module list
        self.modules.insert((filename.into(), base));
    }

  /// Remove the module loaded at `base` in the target process from our
    /// module list
    fn unregister_module(&mut self, base: usize) {
        let mut to_remove = None;

        // Find the corresponding module to this base
        for module in self.modules.iter() {
            if module.1 == base {
                to_remove = Some(module.clone());
            }
        }

        if let Some(to_remove) = to_remove {
            if self.breakpoint_bounds.contains_key(&to_remove.0) {
                // If there are breakpoints in this module, unregister those too

                // Get minimum and maximum offsets into the module where
                // breakpoints are applied
                let minmax = self.breakpoint_bounds[&to_remove.0];

                let start_addr = base + minmax.0;
                let end_addr   = base + minmax.1;

                // Remove any breakpoints which are present in this range
                self.breakpoints.retain(|&k, _| {
                    k < start_addr || k > end_addr
                });
            }

            // Remove the module and breakpoint info for the module
            self.modules.remove(&to_remove);
        } else {
            // Got unregister module for unknown DLL
            // Our database is out of sync with reality
            panic!("Unexpected library unload of base 0x{:x}\n", base);
        }
    }

}


/* Helpers */

pub fn stdio_flush() {
    stdout().flush().expect("Failed to flush stdout");
    stderr().flush().expect("Failed to flush stderr");
}

/* Get elapsed time in seconds */
fn elapsed_from(start: &Instant) -> f64 {
    unimplemented!();
    let dur = start.elapsed();
    dur.as_secs() as f64 + dur.subsec_nanos() as f64 / 1_000_000_000.0
}

pub fn test_unimplemented() {
    unimplemented!();
}
