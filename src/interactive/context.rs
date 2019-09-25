use std::fmt;

use crate::interactive::tabs::TabsState;

const ASNHEADER: [&'static str; 3] = [ "Address", "Opcode", "Ins" ];


const INSTEST: [(usize, &'static str); 10] = [
    ( 0x40000, "mov eax, [ecx]" ),
    ( 0x40000, "mov eax, [ecx]" ),
    ( 0x40000, "mov eax, [ecx]" ),
    ( 0x40000, "mov eax, [ecx]" ),
    ( 0x40000, "mov eax, [ecx]" ),
    ( 0x40000, "mov eax, [ecx]" ),
    ( 0x40000, "mov eax, [ecx]" ),
    ( 0x40000, "mov eax, [ecx]" ),
    ( 0x40000, "mov eax, [ecx]" ),
    ( 0x40000, "mov eax, [ecx]" ),
];

const INS_VEC: [Ins; 15] = [
    Ins { addr: 0x40000, instruction: "mov eax, [ecx]" },
    Ins { addr: 0x40000, instruction: "mov eax, [ecx]" },
    Ins { addr: 0x40000, instruction: "mov eax, [ecx]" },
    Ins { addr: 0x40000, instruction: "mov eax, [ecx]" },
    Ins { addr: 0x40000, instruction: "mov eax, [ecx]" },
    Ins { addr: 0x40000, instruction: "mov eax, [ecx]" },
    Ins { addr: 0x40000, instruction: "mov eax, [ecx]" },
    Ins { addr: 0x40000, instruction: "mov eax, [ecx]" },
    Ins { addr: 0x40000, instruction: "mov eax, [ecx]" },
    Ins { addr: 0x40000, instruction: "mov eax, [ecx]" },
    Ins { addr: 0x40000, instruction: "mov eax, [ecx]" },
    Ins { addr: 0x40000, instruction: "mov eax, [ecx]" },
    Ins { addr: 0x40000, instruction: "mov eax, [ecx]" },
    Ins { addr: 0x40000, instruction: "mov eax, [ecx]" },
    Ins { addr: 0x40000, instruction: "mov eax, [ecx]" },
];


#[derive(Clone)]
pub struct Ins<'a> {
    pub addr: usize,
    pub instruction: &'a str,
}

impl<'a> fmt::Debug for Ins<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Addr: {:#x} Instruction: {}", self.addr, self.instruction)
    }
}

pub struct InsState<'a> {
    pub items: Vec<Ins<'a>>,
    pub selected: usize,
}

// impl InsState {
//     fn new(items: Vec<T>) -> InsState {
//         InsState { items, selected: 0 }
//     }
//     fn select_previous(&mut self) {
//         if self.selected > 0 {
//             self.selected -= 1;
//         }
//     }
//     fn select_next(&mut self) {
//         if self.selected < self.items.len() - 1 {
//             self.selected += 1
//         }
//     }
// }

pub struct ListState<I> {
    pub items: Vec<I>,
    pub selected: usize,
}

impl<I> ListState<I> {
    fn new(items: Vec<I>) -> ListState<I> {
        ListState { items, selected: 0 }
    }
    fn select_previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }
    fn select_next(&mut self) {
        if self.selected < self.items.len() - 1 {
            self.selected += 1
        }
    }
}

pub struct Context<'a> {
    pub title: &'a str,
    pub should_quit: bool,
    pub tabs: TabsState<'a>,
    pub show_src: bool,
    pub disass: ListState<Ins<'a>>,
    // pub disass: InsState<Ins>,
    // pub disass: Vec<Ins<'a>>,
}

impl<'a> Context<'a> {
    pub fn new(title: &'a str) -> Context<'a> {
        Context {
            title,
            should_quit: false,
            tabs: TabsState::new(vec!["Process 0", "Process 1"]),
            show_src: true,
            disass: ListState::new(INS_VEC.iter().cloned().collect()),
            // disass: INS_VEC,
        }
    }

    pub fn on_up(&mut self) {
        self.disass.select_previous();
    }

    pub fn on_down(&mut self) {
        self.disass.select_next();
    }

    pub fn on_right(&mut self) {
        self.tabs.next();
    }

    pub fn on_left(&mut self) {
        self.tabs.previous();
    }

    pub fn on_key(&mut self, c: char) {
        match c {
            'q' => {
                self.should_quit = true;
            }
            _ => {}
        }
    }

    pub fn on_tick(&mut self) {
        let tick = 0;
    }
}
