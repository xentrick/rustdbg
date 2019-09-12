# RustDBG

A process debugger written in Rust.

# Structure Ideas

## Modules

- inferior (inferior data type)
- process start (could be linked to inferior)
- breakpoints
- debug

## Operations

- breakpoints
- stepping
- expression evaluation
- backtrace (stack unwinding)

## Features

- register
- memory writes
- starting a process
- breakpoints
- conditions
- Inferior Process structure for knowledge about the process
- general cmd line debug functions

## To Implement

- [ ] Breakpoint
 - [ ] Software
 - [ ] Hardware
- [ ] Symbols
  - [ ] Source Code
- [ ] Directive for registers
  - [ ] Set breakpoint with register `break $eip`
- [ ] Display Layout
  - [ ] Memory View
  - [ ] Disassembly
  - [ ] Registers
  - [ ] Stack
  - [ ] Source Code

## Things to account for

- [ ] Carry error code in `InferiorState` (fat enum)
- [ ] Manage debugee threads
  - [ ] check for new threads on signal and add to DB
- [ ] Parse args correctly
- [ ] Parse settings into linefeed directive so UI knows about it. 
- [x] Good practice to flush stdio  prior to fork.

## Needs Testing

- [ ] Ctrl+c to break out of `wait`


## vfork 

- [ ] Retain a copy of env varaibles since the child will replace the value of environ. If we arevforked, we have to restore it.

## Random features

- [ ] Allow user to select `cwd` before fork.

# Execution Flow

1. Fork process.


# Potential Implenentation Clones

## GDB

### Interfaces

- User
- Symbols
- Target

## Radare2

- Plugin architecture.
- Bulk split between io/reg/bp/debug

# Random notes

## Breakpoints

* Hardware Breakpoints
  - DR0-DR3: registers for writing addr
  - DR4/DR6: debug status register
  - DR5/DR7: debug control register
    * Break on reading, writing, or executing
* Software breakpoint
  - Rewrite next instruction with Interrupt
  - Replace first byte with `0xCC` and store real one
    ```
    55            push %rbp
    48 89 e5      mov %rsp, %rsp
    48 83 ec 10   sub $0x10,%rsp

    55            push %rbp
    cc 89 e5      mov %rsp, %rsp
    48 83 ec 10   sub $0x10,%rsp
    ```

## Interrupts

* Traps (SIGTRAP)
* Operating system registers interrupt handlers
* Trigger then handler invoked
* `waitpid(m_pid, 0, 0)`
* break `main()`
  - 
  
## Stepping

- Single Step: `ptrace(PTRACE_SINGLESTEP, debuggee_pid, nullptr, nullptr)`
- Step out. Set BP at return addr
- Step Over.
  * BP at return addr
  * AND a BP at next instruction

## Registers

* `ptrace(PTRACE_GETREGS, pid, nullptr, &regs)`

## Read/Write Memory

* Read/Write. One `WORD` at a time
  ```
  auto date = ptrace(PTRACE_PEEKDATA, m_pid, address, nullptr);
  data |= 1
  ptrace(PTRACE_POKEDATA, m_pid, address, data)
  ```
* process_vm_readv. Multi `WORD` read/writes. Probably better

## Multi-Thread 

* Trap Clone: `ptrace(PTRACE_SETOPTIONS, m_pid, nullptr, PTRACE_O_TRACECLONE)`
  ```
  case (SIGTRAP | (PTRACE_EVENT_CLONE << 8))
  // get the new thread ID
  unsigned long event_message = 0;
  ptrace(PTRACE_GETEVENTMSG, pid, nullptr, message);
  // handle creation
  //...
  ```
  
## Shared Library

* BP on library not loaded
```
typdef struct {
  Elf64_Sxword d_tag;    /* entry tag value */
  union {
    Elf64_Xword d_val;
    Elf64_addr d_ptr;
  } d_un
} Elf64_Dyn
```

- struct `r_debug`
  - `link_map` navigate loaded SOs
  - `ElfW(Addr) r_brk`: Address of function with SO is loaded/unloaded
    - Set SW breakpoint, to populate `link_map`

## Ptrace info

```
PTRACE_TRACEME
PTRACE_PEEKDATA
PTRACE_POKEDATA
PTRACE_GETREGS
PTRACE_SETREGS
```



