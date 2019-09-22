use std::io;

use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::canvas::{Canvas, Line, Map, MapResolution, Rectangle};
use tui::widgets::{
    Axis, BarChart, Block, Borders, Chart, Dataset, Gauge, List, Marker, Paragraph, Row,
    SelectableList, Sparkline, Table, Tabs, Text, Widget,
};
use tui::{Frame, Terminal};

use crate::interactive::app::App;

pub fn draw<B: Backend>(terminal: &mut Terminal<B>, app: &App) -> Result<(), io::Error> {
    terminal.draw(|mut f| {
        let chunks = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(f.size());
        Tabs::default()
            .block(Block::default().borders(Borders::ALL).title(app.title))
            .titles(&app.tabs.titles)
            .style(Style::default().fg(Color::Green))
            .highlight_style(Style::default().fg(Color::Yellow))
            .select(app.tabs.index)
            .render(&mut f, chunks[0]);
        match app.tabs.index {
            0 => draw_context(&mut f, &app, chunks[1]),
            // 1 => draw_second_tab(&mut f, &app, chunks[1]),
            _ => {}
        };
    })
}

fn draw_context<B>(f: &mut Frame<B>, app: &App, area: Rect)
where B: Backend,
{
    let chunks = Layout::default()
        .constraints(
            [
                Constraint::Min(7),
                Constraint::Length(7),
            ].as_ref()).split(area);
    draw_disass(f, app, chunks[0]);
}

fn draw_disass<B>(f: &mut Frame<B>, app: &App, area: Rect)
where B: Backend,
{
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
        .direction(Direction::Horizontal)
        .split(area);
    let selected_style = Style::default().fg(Color::Yellow).modifier(Modifier::BOLD);
    let normal_style = Style::default().fg(Color::White);
    let header = ["Address", "Instruction"];
    let rows = app.disass.items.iter().map(|s| {
        Row::StyledData(vec![s.addr, s.addr].into_iter(), normal_style)
        // Row::StyledData(s.into_iter(), normal_style)
    });
    Table::new(header.into_iter(), rows)
        .block(Block::default().title("Disassembly").borders(Borders::ALL))
        .header_style(Style::default().fg(Color::Yellow))
        .widths(&[24, 48])
        .render(f, chunks[0]);
    let header = ["Address", "Instruction"];
}
