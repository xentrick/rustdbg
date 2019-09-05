pub mod commands;
pub mod completer;

use linefeed::{Interface, ReadResult};
use linefeed::command::COMMANDS;
use linefeed::inputrc::parse_text;
use std::io;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::path::Path;

use self::commands::*;
use self::completer::*;
use debug;

const HISTORY_FILE: &str = ".rdbg_history";

// https://github.com/murarth/linefeed/blob/master/examples/demo.rs

pub fn main() -> io::Result<()> {
    println!("Initializing rustdbg debugger. Written by xentrick");

    let interface = Arc::new(Interface::new("rustdbg")?);
    interface.set_prompt("rdbg> ")?;

    if let Err(e) = interface.load_history(HISTORY_FILE) {
        if e.kind() == io::ErrorKind::NotFound {
            println!("History file {} doesn't exist, not loading history.", HISTORY_FILE);
        } else {
            eprintln!("Could not load history file {}: {}", HISTORY_FILE, e);
        }
    }
    while let ReadResult::Input(line) = interface.read_line()? {
        if !line.trim().is_empty() {
            interface.add_history_unique(line.clone());
        }

        let (cmd, args) = split_first_word(&line);

        match cmd {
            "run" => {
                let mut inf = debug::start(Path::new("/home/nmavis/dev/rustdbg/tests/elf/hello_world"), &[]).unwrap();
                debug::breakpoint::set_bp(inf, 0x55555555513d);
                debug::continue_exec(&mut inf);
                //debug::start(Path::new("/home/nmavis/dev/rustdbg/tests/rust/target/debug/hello_world"), &[]);
            }
            "help" => {
                println!("rustdbg commands:\n");
                for &(cmd, help) in RDBG_COMMANDS {
                    println!("  {:15} - {}", cmd, help);
                }
                println!();
            }
           "list-commands" => {
                for cmd in COMMANDS {
                    println!("{}", cmd);
                }
            }
            "list-variables" => {
                for (name, var) in interface.lock_reader().variables() {
                    println!("{:30} = {}", name, var);
                }
            }
            "history" => {
                let w = interface.lock_writer_erase()?;

                for (i, entry) in w.history().enumerate() {
                    println!("{}: {}", i, entry);
                }
            }
            "save-history" => {
                if let Err(e) = interface.save_history(HISTORY_FILE) {
                    eprintln!("Could not save history file {}: {}", HISTORY_FILE, e);
                } else {
                    println!("History saved to {}", HISTORY_FILE);
                }
            }
            "quit" => break,
            "set" => {
                let d = parse_text("<input>", &line);
                interface.evaluate_directives(d);
            }
            _ => println!("read input: {:?}", line)
        }
    }

    println!("Goodbye.");

    Ok(())
}

fn split_first_word(s: &str) -> (&str, &str) {
    let s = s.trim();

    match s.find(|ch: char| ch.is_whitespace()) {
        Some(pos) => (&s[..pos], s[pos..].trim_start()),
        None => (s, "")
    }
}
