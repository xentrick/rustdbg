use std::io;
use std::fmt;
use std::sync::Arc;

use linefeed::{Interface, ReadResult};
use linefeed::command::COMMANDS;
use linefeed::inputrc::parse_text;
use linefeed::terminal::DefaultTerminal;


use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::canvas::{Canvas, Line, Map, MapResolution, Rectangle};
use tui::widgets::{
    Axis, BarChart, Block, Borders, Chart, Dataset, Gauge, List, Marker, Paragraph, Row,
    SelectableList, Sparkline, Table, Tabs, Text, Widget,
};
use tui::{Frame, Terminal};

use crate::interactive::context::Context;
use crate::inferior::Inferior;

const LIST_TEST: [&'static str; 10] = [
    "test",
    "test",
    "test",
    "test",
    "test",
    "test",
    "test",
    "test",
    "test",
    "test",
];

#[derive(Debug)]
enum InsFormat<'a> {
    Address(usize),
    Assembly(&'a str),
}

impl<'a> fmt::Display for InsFormat<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            InsFormat::Address(u) => write!(f, "{:#x}", u),
            InsFormat::Assembly(s) => write!(f, "{}", s),
        }
    }
}

pub fn draw<B: Backend>(terminal: &mut Terminal<B>, app: &Context, inf: &Inferior, linefeed: &Arc<Interface<DefaultTerminal>>) -> Result<(), io::Error> {
    terminal.draw(|mut f| {
        let chunks = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(f.size());
        Tabs::default()
            .block(Block::default().borders(Borders::ALL).title(app.title))
            .titles(&app.tabs.titles)
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Blue))
            .select(app.tabs.index)
            .render(&mut f, chunks[0]);
        match app.tabs.index {
            0 => draw_context(&mut f, &app, chunks[1], inf, linefeed),
            // 1 => draw_second_tab(&mut f, &app, chunks[1]),
            _ => {}
        };
    })
}

fn draw_context<B>(f: &mut Frame<B>, app: &Context, area: Rect, inf: &Inferior, linefeed: &Arc<Interface<DefaultTerminal>>)
where B: Backend,
{
    let chunks = Layout::default()
        .constraints(
            [
                Constraint::Min(7),
                Constraint::Length(7),
            ].as_ref()).split(area);
    draw_disass(f, app, chunks[0], inf);
    draw_cli(f, app, chunks[1], linefeed);
}

fn draw_disass<B>(mut f: &mut Frame<B>, app: &Context, area: Rect, inf: &Inferior)
where B: Backend,
{
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
        .direction(Direction::Horizontal)
        .split(area);

    // Draw instruction context
    let selected_style = Style::default().fg(Color::Green).modifier(Modifier::BOLD);
    let normal_style = Style::default().fg(Color::White);
    let header = ["Address", "Instruction"];
    let rows = app.disass.items.iter().enumerate().map(|(i, item)| {
        if i == app.disass.selected {
            Row::StyledData(vec![InsFormat::Address(item.addr), InsFormat::Assembly(item.instruction)].into_iter(), selected_style)
        } else {
            Row::StyledData(vec![InsFormat::Address(item.addr), InsFormat::Assembly(item.instruction)].into_iter(), normal_style)
            // Row::StyledData(vec![item.addr, item.addr].into_iter(), normal_style)
        }
    });
    Table::new(header.into_iter(), rows)
        .block(Block::default().borders(Borders::ALL).title("Disassembly"))
        .header_style(Style::default().fg(Color::White))
        .widths(&[16, 24])
        .render(&mut f, chunks[0]);

    // Draw Register / Stack
    let panel = Layout::default()
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .direction(Direction::Vertical)
        .split(chunks[1]);

    Paragraph::new([Text::raw( "Testing" )].iter())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Registers")
                .title_style(Style::default().fg(Color::Magenta).modifier(Modifier::BOLD)),
        )
        .wrap(true)
        .render(f, panel[0]);

    Paragraph::new([Text::raw( "Testing" )].iter())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Stack")
                .title_style(Style::default().fg(Color::Magenta).modifier(Modifier::BOLD)),
        )
        .wrap(true)
        .render(f, panel[1]);
}



fn draw_cli<B>(mut f: &mut Frame<B>, app: &Context, area: Rect, linefeed: &Arc<Interface<DefaultTerminal>>)
where B: Backend,
{
    Paragraph::new([Text::raw( "Testing" )].iter())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Cli")
                .title_style(Style::default().fg(Color::Magenta).modifier(Modifier::BOLD)),
        )
        .wrap(true)
        .render(f, area);

}
