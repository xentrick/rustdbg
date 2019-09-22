use crate::interactive::util::TabsState;

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

pub struct Ins<'a> {
    pub addr: usize,
    pub instruction: &'a str,
}

pub struct InsState {
    pub items: Vec<Ins>,
    pub selected: usize,
}

impl InsState {
    fn new(items: Vec) -> InsState {
        InsState { items, selected: 0 }
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

pub struct App<'a> {
    pub title: &'a str,
    pub should_quit: bool,
    pub tabs: TabsState<'a>,
    pub show_src: bool,
    pub disass: InsState<Ins>,
    // pub disass: Vec<Ins<'a>>,
}

impl<'a> App<'a> {
    pub fn new(title: &'a str) -> App<'a> {
        App {
            title,
            should_quit: false,
            tabs: TabsState::new(vec!["Process 0", "Process 1"]),
            show_src: true,
            disass: InsState::new(INSTEST.to_vec()),
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
