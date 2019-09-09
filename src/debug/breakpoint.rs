use crate::inferior::*;
use crate::debug::*;
use nix::unistd::Pid;




pub fn set(pid: Pid, addr: u64) -> Result<Breakpoint, Error> {
    println!("Setting breakpoint at 0x{:x}", addr);
    let aligned_address = addr & !0x7u64;
    let mut bp = Breakpoint {
        enabled: false,
        shift : (addr - aligned_address) * 8,
        aligned_address: InferiorPointer(aligned_address),
        target_address: InferiorPointer(addr),
        saved: peek(pid, InferiorPointer(aligned_address))?,
    };

    println!("EIP[0]: {:#x}", bp.saved.swap_bytes());

    let mut modinstruction = bp.saved;
    modinstruction &= !0xFFi64 << bp.shift;
    modinstruction |= 0xCCi64 << bp.shift;
    write(pid, bp.aligned_address, modinstruction);

    bp.enabled = true;
    Ok(bp)
}
