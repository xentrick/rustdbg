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
pub fn main() -> io::Result<()> {
    // Intialize fresh rustdbg interface
    let menu = Menu::new().expect("Unable to initialize rustdbg interface.");
    // Start command loop to get user input.
    menu.cmdloop().expect("Unable to start command loop.");
    Ok(())
}


