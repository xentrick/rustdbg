/* rustdbg command list */

pub static RDBG_COMMANDS: &[(&str, &str)] = &[
    ("help", "Display help"),
    ("quit", "Exit rdbg"),
    ("load", "Load a file"),
    ("run", "Execute a file"),
    ("execute", "Execute a file"),
    ("break", "Set a breakpoint"),
    ("continue", "Continue execution"),
    ("hexdump", "Dump Hex"),
    ("registers", "Show register information for inferior"),
    ("symbols", "Show symbols for inferior"),
];
