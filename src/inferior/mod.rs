/* Inferior Process Module */

use nix::unistd::Pid;
use libc::c_void;
use std::ops::{Add, Sub};
use std::fmt;
use std::cell::RefCell;

/* Enums */

#[derive(Copy, Clone)]
pub struct InferiorPointer(pub u64);

impl InferiorPointer {
    pub fn as_voidptr(&self) -> * mut c_void {
        let &InferiorPointer(u) = self;
        u as * mut c_void
    }
}

impl fmt::LowerHex for InferiorPointer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val = self.0;
        write!(f, "{:#x}", val)
    }
}

impl Add<i64> for InferiorPointer {
    type Output = InferiorPointer;
    fn add(self, rhs: i64) -> InferiorPointer {
        let InferiorPointer(u) = self;
        if rhs >= 0 {
            InferiorPointer(u + rhs as u64)
        } else {
            InferiorPointer(u - rhs as u64)
        }
    }
}

impl Sub<i64> for InferiorPointer {
    type Output = InferiorPointer;
    fn sub(self, rhs: i64) -> InferiorPointer {
        let InferiorPointer(u) = self;
        if rhs >= 0 {
            InferiorPointer(u - rhs as u64)
        } else {
            InferiorPointer(u + rhs as u64)
        }
    }
}

#[derive(Copy, Clone)]
pub enum InferiorState {
    Running,
    Stopped,
    Sleeping,
    Zombie,
    Dead,
    Raised,  /* Traps (Ex: Single step) */
    None,
}

#[derive(Copy, Clone)]
pub enum SignalMode {
    SignalIgnore = 0,
    SignalCont = 1,
    SignalSkip = 2
}

#[derive(Copy, Clone)]
pub enum RecoilMode {
    RecoilNone = 0,
    RecoilStep,
    RecoilContinue
}

#[derive(Copy, Clone)]
pub enum ReasonType {
    ReasonDead = -1,
    ReasonNone = 0,
    ReasonSignal,
    ReasonSegfault,
    ReasonBreakpoint,
    ReasonTracepoint,
    ReasonCond,
    ReasonReaderr,
    ReasonStep,
    ReasonAbort,
    ReasonWriterr,
    ReasonDivbyzero,
    ReasonIllegal,
    ReasonUnknown,
    ReasonErrork,
    ReasonNewPid,
    ReasonNewTid,
    ReasonNewLib,
    ReasonExitPid,
    ReasonExitTid,
    ReasonExitLib,
    ReasonTrap,
    ReasonSwi,
    ReasonInt,
    ReasonFpu,
    ReasonUsersups,
}

/* Structures */

#[derive(Copy, Clone)]
pub struct Reason {
    r#type: u32,
    tid: u32,
    signum: u32,
    bp_addr: u64,
    timestamp: u64,
    addr: u64,
    ptr: u64,
}

#[derive(Copy, Clone)]
pub struct MemoryMap {
    start: u64,
    size: u64,
    in_use: bool,
}

#[derive(Copy, Clone)]
pub struct Breakpoint {
    pub enabled: bool,
    pub shift : u64,
    pub target_address  : InferiorPointer,
    pub aligned_address : InferiorPointer,
    pub saved : i64,
}

// #[derive(Clone)]
// pub struct BreakPointMap {
//    pub list: RefCell<Breakpoint>,
// }

#[derive(Clone)]
pub struct Inferior {
    /* Process Information */
    //main_pid: Option<Pid>,
    pub pid: Pid,
    //tid: Option<Pid>,
    //forked_pid: Option<Pid>,
    pub attached: bool,

    /* Process State */
    pub state: InferiorState,
    //pub mem: MemoryMap,

    pub bpmap: Vec<Breakpoint>,
    /* List of breakpoints */

    //steps: u32,
    //reason: Reason,
    //recoil_mode: RecoilMode,
    //stopaddr: u64,
}

impl Default for Inferior {
    fn default() -> Inferior {
        Inferior {
            pid: Pid::this(),
            attached: false,
            state: InferiorState::None,
            bpmap: Vec::with_capacity(10),
        }
    }
}
