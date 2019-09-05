extern crate rustdbg;

use rustdbg::*;
use std::path::Path;

fn main() {
    interactive::main().expect("Failed to run interactive menu");
}

fn test() {
    let mut inf = debug::start(Path::new("/home/nmavis/dev/rustdbg/tests/elf/hello_world"), &[]).unwrap();
    debug::breakpoint::set_bp(inf, 0x55555555513d);
    debug::continue_exec(&mut inf);
    //debug::start(Path::new("/home/nmavis/dev/rustdbg/tests/rust/target/debug/hello_world"), &[]);
}
