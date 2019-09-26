pub mod console;
pub mod context;
pub mod commands;
pub mod completer;
pub mod tabs;
pub mod ui;
pub mod util;
mod fmt;

use std::io;

use self::console::Menu;

// Create the rustdbg interface on startup
pub fn main() -> Result<(), failure::Error> {
    // Intialize fresh rustdbg interface
    // let rdbg = Menu::new().unwrap().expect("Unable to initialize rustdbg interface.");
    let rdbg: Menu<'_, B> = Menu::new().unwrap();
    // Start command loop to get user input.
    rdbg.cmdloop();
    Ok(())
}


