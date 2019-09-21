#[allow(dead_code)]
#[allow(unused)]

pub mod commands;
pub mod completer;
pub mod ui;
pub mod app;
pub mod util;
mod fmt;

use linefeed::{Interface, ReadResult};
use linefeed::command::COMMANDS;
use linefeed::inputrc::parse_text;
use std::io;
use std::sync::Arc;
//use std::thread;
//use std::time::Duration;
//use std::path::Path;
//use std::u64;
use ansi_term::Color;

use crate::interactive::commands::*;
use crate::interactive::completer::DbgCompleter;
use crate::inferior::Inferior;

use std::unimplemented;

const HISTORY_FILE: &str = ".rdbg_history";

// https://github.com/murarth/linefeed/blob/master/examples/demo.rs

pub fn main() -> io::Result<()> {
    println!("Initializing rustdbg debugger. Written by xentrick");

    let interface = Arc::new(Interface::new("rustdbg")?);
    interface.set_completer(Arc::new(DbgCompleter));

    let green = Color::Green.bold();

    interface.set_prompt(&format!("\x01{prefix}\x02{text}\x01{suffix}\x02",
             prefix=green.prefix(),
             text="rdbg> ",
             suffix=green.suffix()))?;

    if let Err(e) = interface.load_history(HISTORY_FILE) {
        if e.kind() == io::ErrorKind::NotFound {
            println!("History file {} doesn't exist, not loading history.", HISTORY_FILE);
        } else {
            eprintln!("Could not load history file {}: {}", HISTORY_FILE, e);
        }
    }

    let mut inf = Inferior::new();

    while let ReadResult::Input(line) = interface.read_line()? {
        if !line.trim().is_empty() {
            interface.add_history_unique(line.clone());
        }

        let (cmd, _args) = split_first_word(&line);
        let debug_target = String::from("/home/nmavis/dev/rustdbg/tests/elf/hello_world");
        let debug_args = &[];

        match cmd {
            "test" => {
                inf.start(debug_target, debug_args);
                //Inferior::breakpoint::set(inf.pid, 0x55555555513d);
                //debug::resume(inf);
                //debug::start(Path::new("/home/nmavis/dev/rustdbg/tests/rust/target/debug/hello_world"), &[]);
            }
            "run" => inf.start(_args.to_string(), debug_args),
            "continue" => inf.resume(),
            "break" => {
                let bpaddr = _args.split_whitespace().collect();
                inf.set_breakpoint(bpaddr);
            },
            "registers" => println!("{:#x?}", inf.registers()),
            "memory" => inf.show_memory_map(),
            // "files" => inf.files(),
            "env" => println!("{:#?}", inf.env),
            "pcode" => unimplemented!(),
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

    interface.save_history(HISTORY_FILE)?;
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
