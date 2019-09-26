use ansi_term::Color;
use linefeed::{Interface, ReadResult};
use linefeed::command::COMMANDS;
use linefeed::inputrc::parse_text;
use linefeed::terminal::DefaultTerminal;
use std::io;
use std::io::StdoutLock;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use std::unimplemented;
use structopt::StructOpt;
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::{Backend, TermionBackend};
use tui::terminal::Terminal;
//use tui::Terminal;

//use std::thread;
//use std::time::Duration;
//use std::path::Path;
//use std::u64;

use crate::inferior::Inferior;
use crate::interactive::context::Context;
use crate::interactive::commands::*;
use crate::interactive::completer::DbgCompleter;
use crate::interactive::util::{split_first_word};
use crate::interactive::util::event::{Config, Event, Events};
use crate::interactive::ui;
//use self::app::{ui, App};
//use self::util::event::{Config, Event, Events};


const HISTORY_FILE: &str = ".rdbg_history";

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(long = "tick-rate", default_value = "250")]
    tick_rate: u64,
    #[structopt(long = "log")]
    log: bool,
}

struct ContextView<'a>
{
    cli: Cli,
    events: Events,
    app: Context<'a>,
    terminal: Terminal,
}

impl<'a> ContextView<'a> {
    fn new() -> Result<ContextView<'a>, failure::Error> {
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

        Ok(ContextView {
            cli: cli,
            events: events,
            terminal: terminal,
            app: app,
        })
    }

    fn show(&mut self, menu: &Menu) -> Result<(), failure::Error> {
        loop {
            //ui::draw(&mut self.terminal, &self.app, &self.inf, &self.linefeed)?;
            ui::draw(&mut self.terminal, &self.app, &menu.inferior, &menu.linefeed)?;
            match self.events.next()? {
                Event::Input(key) => match key {
                    Key::Char(c) => {
                        self.app.on_key(c);
                    }
                    Key::Up => {
                        self.app.on_up();
                    }
                    Key::Down => {
                        self.app.on_down();
                    }
                    Key::Left => {
                        self.app.on_left();
                    }
                    Key::Right => {
                        self.app.on_right();
                    }
                    _ => {}
                },
                Event::Tick => {
                    self.app.on_tick();
                }
            }
            if self.app.should_quit {
                break;
            }
        }

        self.terminal.clear();
        Ok(())
    }
}

fn oldcontextfn(linefeed: &Arc<Interface<DefaultTerminal>>, inf: &Inferior) -> Result<(), failure::Error> {
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

pub struct Menu<'a> {
    linefeed: Arc<Interface<DefaultTerminal>>,
    context: ContextView<'a>,
    inferior: Inferior,
}


impl<'a> Menu<'a> {
    // Create `Menu` object. Implement as Result for errors
    pub fn new() -> Result<Menu<'a>, failure::Error> {
        // Initialize thread safe `Interface`
        let interface = Arc::new(Interface::new("rustdbg")?);
        interface.set_completer(Arc::new(DbgCompleter));

        // Set prompt
        interface.set_prompt(&format!("\x01{prefix}\x02{text}\x01{suffix}\x02",
                                      prefix=Color::Green.bold().prefix(),
                                      text="rdbg> ",
                                      suffix=Color::Green.bold().suffix()))?;
        let rdbg = Menu { linefeed: interface, context: ContextView::new()?, inferior: Inferior::new() };
        // Load History and return `Menu` structure
        rdbg.load_history();
        Ok(rdbg)
    }

    pub fn load_history(&mut self) {
        if let Err(e) = self.linefeed.load_history(HISTORY_FILE) {
            if e.kind() == io::ErrorKind::NotFound {
                println!("History file {} doesn't exist, not loading history.", HISTORY_FILE);
            } else {
                eprintln!("Could not load history file {}: {}", HISTORY_FILE, e);
            }
        }
    }

    // https://github.com/murarth/linefeed/blob/master/examples/demo.rs
    pub fn cmdloop(&mut self) -> Result<(), failure::Error> {
        while let ReadResult::Input(line) = self.linefeed.read_line()? {
            if !line.trim().is_empty() {
                self.linefeed.add_history_unique(line.clone());
            }

            let (cmd, _args) = split_first_word(&line);
            let debug_target = String::from("/home/nmavis/dev/rustdbg/tests/elf/hello_world");
            let debug_args = &[];

            match cmd {
                "context" => {
                    if let Err(e) = self.context.show(&self.linefeed, &self.inferior) {
                        println!("Context Error: {}", e);
                    }
                },
                "test" => {
                    self.inferior.start(debug_target, debug_args);
                    //Inferior::breakpoint::set(inf.pid, 0x55555555513d);
                    //debug::resume(inf);
                    //debug::start(Path::new("/home/nmavis/dev/rustdbg/tests/rust/target/debug/hello_world"), &[]);
                },
                "run" => {
                    if _args.is_empty() { println!("Please provide a process path to debug"); }
                    else if Path::new(_args).is_file() { self.inferior.start(_args.into(), debug_args); }
                    else { println!("Invalid path to inferior."); }
                }
                "continue" => self.inferior.resume(),
                "break" => {
                    let bpaddr = _args.split_whitespace().collect();
                    self.inferior.set_breakpoint(bpaddr);
                },
                "registers" => println!("{:#x?}", self.inferior.registers()),
                "memory" => self.inferior.show_memory_map(),
                // "files" => inf.files(),
                "env" => println!("{:#?}", self.inferior.env),
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
                    for (name, var) in self.linefeed.lock_reader().variables() {
                        println!("{:30} = {}", name, var);
                    }
                }
                "history" => {
                    let w = self.linefeed.lock_writer_erase()?;

                    for (i, entry) in w.history().enumerate() {
                        println!("{}: {}", i, entry);
                    }
                }
                "save-history" => {
                    if let Err(e) = self.linefeed.save_history(HISTORY_FILE) {
                        eprintln!("Could not save history file {}: {}", HISTORY_FILE, e);
                    } else {
                        println!("History saved to {}", HISTORY_FILE);
                    }
                }
                "quit" => break,
                "set" => {
                    let d = parse_text("<input>", &line);
                    self.linefeed.evaluate_directives(d);
                }
                _ => println!("read input: {:?}", line)
            }
        }

        self.linefeed.save_history(HISTORY_FILE)?;
        println!("Goodbye.");

        Ok(())
    }
}


