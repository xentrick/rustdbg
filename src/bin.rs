extern crate rustdbg;

use rustdbg::*;
use std::path::Path;

fn main() {
    let mut inf = debug::start(Path::new("/home/nmavis/dev/rustdbg/tests/elf/hello_world"), &[]).unwrap();
    debug::breakpoint::set_bp(inf, 0x55555555513d);
    debug::continue_exec(&mut inf);
    //debug::start(Path::new("/home/nmavis/dev/rustdbg/tests/rust/target/debug/hello_world"), &[]);
}
