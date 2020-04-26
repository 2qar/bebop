extern crate tui;
extern crate termion;

use std::io;
use std::fs;
use std::fs::DirEntry;

use termion::raw::IntoRawMode;
use termion::event::{Event, Key};
use termion::input::TermRead;

use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::{List, ListState, Block, Borders, Text};
use tui::layout::{Layout, Constraint, Direction};
use tui::style::{Color, Style, Modifier};

fn main() -> Result<(), io::Error> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;
    terminal.hide_cursor()?;

    let artist_paths: Vec<DirEntry> = fs::read_dir("/home/tucker/Music")?
        .filter(|de| de.as_ref().unwrap().path().is_dir())
        .map(|de| de.unwrap())
        .collect();

    let mut state = ListState::default();
    state.select(Some(0));
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
                .highlight_style(Style::default().bg(Color::Green).modifier(Modifier::BOLD))
                .highlight_symbol(">>");
            f.render_stateful_widget(block, chunks[0], &mut state);
        })?;

        // FIXME make it less panicky looking
        match io::stdin().events().next().unwrap().unwrap() {
            Event::Key(Key::Char('q')) => {
                terminal.clear()?;
                return Ok(());
            },
            Event::Key(Key::Char('j')) => {
                let selected = state.selected().unwrap();
                if selected > artist_paths.len()-2 {
                    state.select(Some(0));
                } else {
                    state.select(Some(selected + 1));
                }
            }
            Event::Key(Key::Char('k')) => {
                let selected = state.selected().unwrap();
                if selected == 0 {
                    state.select(Some(artist_paths.len()-1));
                } else {
                    state.select(Some(selected - 1));
                }
            }
            _ => (),
        }
    }
}
