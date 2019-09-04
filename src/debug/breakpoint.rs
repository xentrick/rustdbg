use crate::inferior::*;
use crate::debug::*;

//#[derive(Copy, Clone)]
struct Breakpoint {
    shift : u64,
    target_address  : InferiorPointer,
    aligned_address : InferiorPointer,
    original_breakpoint_word : i64
}

pub fn set_bp(proc: Inferior, addr: u64) {
    println!("Setting breakpoint at 0x{:x}", addr);
    let aligned_address = addr & !0x7u64;
    let bp = Breakpoint {
        shift : (addr - aligned_address) * 8,
        aligned_address: InferiorPointer(aligned_address),
        target_address: InferiorPointer(addr),
        original_breakpoint_word: peek(proc.pid, InferiorPointer(aligned_address))
    };

    /*set(inferior, bp);

    unsafe {
        global_breakpoint = bp;
    }*/

}
