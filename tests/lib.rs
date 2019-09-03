extern crate rustdbg;
use std::path::Path;

#[test]
fn hello_world () {
    let _debug = rustdbg::debug::start(Path::new("./elf/hello_world"), &[]);
    //assert_eq!(12);
}

fn random_rust () {
    let _debug = rustdbg::debug::start(Path::new("./elf/hello_world"), &[]);

}
