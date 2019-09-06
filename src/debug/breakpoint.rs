use crate::inferior::*;
use crate::debug::*;




pub fn set_bp(proc: Inferior, addr: u64) {
    println!("Setting breakpoint at 0x{:x}", addr);
    let aligned_address = addr & !0x7u64;
    let bp = Breakpoint {
        enabled: false,
        shift : (addr - aligned_address) * 8,
        aligned_address: InferiorPointer(aligned_address),
        target_address: InferiorPointer(addr),
        saved: peek(proc.pid, InferiorPointer(aligned_address))
    };

    println!("EIP[0]: {:#x}", bp.saved.swap_bytes());

    /*set(inferior, bp);

    unsafe {
        global_breakpoint = bp;
    }*/

}
