/* Inferior Process Module */

use nix::unistd::Pid;

/* Enums */

#[derive(Copy, Clone)]
pub enum InferiorState {
    Running,
    Stopped,
    Sleeping,
    Zombie,
    Dead,
    Raised  /* Traps (Ex: Single step) */
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
    ReasonError,
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
pub struct Inferior {
    /* Process Information */
    main_pid: Option<Pid>,
    pid: Option<Pid>,
    tid: Option<Pid>,
    forked_pid: Option<Pid>,

    /* Process State */
    state: InferiorState,

    steps: u32,
    reason: Reason,
    recoil_mode: RecoilMode,
    stopaddr: u64,
}
