extern crate rustdbg;

use rustdbg::*;
use std::path::Path;

fn main() {
    interactive::main().expect("Failed to run interactive menu");
}
