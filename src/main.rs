extern crate tui;
extern crate termion;

use std::io;
use std::fs;
use std::fs::DirEntry;
use std::borrow::Cow;

use termion::raw::IntoRawMode;
use termion::event::{Event, Key};
use termion::input::TermRead;

use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::{List, Block, Borders, Text, Gauge};
use tui::layout::{Layout, Constraint, Direction};
use tui::style::{Color, Style};

fn main() -> Result<(), io::Error> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;
    terminal.hide_cursor()?;

    let mut artist_paths: Vec<DirEntry> = fs::read_dir("/home/tucker/Music")?
        .filter(|de| de.as_ref().unwrap().path().is_dir())
        .map(|de| de.unwrap())
        .collect();

    loop {
        terminal.draw(|mut f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                    Constraint::Percentage(100),
                    Constraint::Percentage(10),
                    ].as_ref()
                )
                .split(f.size());

            let artist_strings = artist_paths.iter().map(|de| {
                de.path().into_os_string().into_string().unwrap()
            });
            let block = List::new(artist_strings.map(|de| Text::raw(de)))
                .block(Block::default().title("Artists").borders(Borders::ALL))
                .highlight_symbol(">>");
            f.render_widget(block, chunks[0]);

            /*
            let block = Gauge::default()
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::White).bg(Color::Black))
                .percent(20);
            f.render_widget(block, chunks[1]);
            */
        })?;

        for e in io::stdin().events() {
            match e.unwrap() {
                Event::Key(Key::Char('q')) => {
                    terminal.clear()?;
                    return Ok(());
                },
                _ => (),
            }
        }
    }
}
