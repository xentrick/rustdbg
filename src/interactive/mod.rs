#[allow(dead_code)]
#[allow(unused)]

pub mod commands;
pub mod completer;
pub mod ui;
pub mod context;
pub mod util;
mod fmt;

use ansi_term::Color;
use linefeed::{Interface, ReadResult};
use linefeed::command::COMMANDS;
use linefeed::inputrc::parse_text;
use linefeed::terminal::DefaultTerminal;
use std::io;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use std::unimplemented;
use structopt::StructOpt;
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::Terminal;

//use std::thread;
//use std::time::Duration;
//use std::path::Path;
//use std::u64;

use crate::inferior::Inferior;
use self::commands::*;
use self::completer::DbgCompleter;
//use self::app::{ui, App};
//use self::util::event::{Config, Event, Events};

use util::event::{Config, Event, Events};
pub use context::Context;

const HISTORY_FILE: &str = ".rdbg_history";

/// Intialize TUI and command line loop to process user input.
pub fn main() -> io::Result<()> {
    // linefeed command loop
    cmdloop().expect("Unable to start command loop.");
    Ok(())
}

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(long = "tick-rate", default_value = "250")]
    tick_rate: u64,
    #[structopt(long = "log")]
    log: bool,
}

fn context(linefeed: &Arc<Interface<DefaultTerminal>>, inf: &Inferior) -> Result<(), failure::Error> {
    let cli = Cli::from_args();

    stderrlog::new().quiet(!cli.log).verbosity(4).init()?;

    let events = Events::with_config(Config {
        tick_rate: Duration::from_millis(cli.tick_rate),
        ..Config::default()
    });

    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let mut app = Context::new("rustdbg");
    loop {
        ui::draw(&mut terminal, &app, &inf, &linefeed)?;
        match events.next()? {
            Event::Input(key) => match key {
                Key::Char(c) => {
                    app.on_key(c);
                }
                Key::Up => {
                    app.on_up();
                }
                Key::Down => {
                    app.on_down();
                }
                Key::Left => {
                    app.on_left();
                }
                Key::Right => {
                    app.on_right();
                }
                _ => {}
            },
            Event::Tick => {
                app.on_tick();
            }
        }
        if app.should_quit {
            break;
        }
    }

    terminal.clear();
    Ok(())
}

// https://github.com/murarth/linefeed/blob/master/examples/demo.rs

pub fn cmdloop() -> io::Result<()> {
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

    // Context Setup
    // 

    let mut inf = Inferior::new();

    while let ReadResult::Input(line) = interface.read_line()? {
        if !line.trim().is_empty() {
            interface.add_history_unique(line.clone());
        }

        let (cmd, _args) = split_first_word(&line);
        let debug_target = String::from("/home/nmavis/dev/rustdbg/tests/elf/hello_world");
        let debug_args = &[];

        match cmd {
            "context" => {
                if let Err(e) = context(&interface, &inf) {
                    println!("Context Error: {}", e);
                }
            },
            "test" => {
                inf.start(debug_target, debug_args);
                //Inferior::breakpoint::set(inf.pid, 0x55555555513d);
                //debug::resume(inf);
                //debug::start(Path::new("/home/nmavis/dev/rustdbg/tests/rust/target/debug/hello_world"), &[]);
            },
            "run" => {
                if _args.is_empty() { println!("Please provide a process path to debug"); }
                else if Path::new(_args).is_file() { inf.start(_args.to_string(), debug_args); }
                else { println!("Invalid path to inferior."); }
            }
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
