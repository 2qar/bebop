extern crate tui;
extern crate termion;

use std::io;

use termion::raw::IntoRawMode;
use termion::event::{Event, Key};
use termion::input::TermRead;

use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::{List, Block, Borders, Text};
use tui::layout::{Layout, Constraint, Direction};
use tui::style::{Color, Style, Modifier};

use bebop::*;

mod player;
use player::Player;

fn main() -> Result<(), io::Error> {
    let mut player = Player::new(0.2);

    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;
    terminal.hide_cursor()?;

    let mut explorer = Explorer::new("/home/tucker/Music")?;
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

            let dir_strings = explorer.selected_dir().entry_strings();
            let block = List::new(dir_strings.iter().map(|de| Text::raw(de)))
                .block(Block::default().title("Artists").borders(Borders::ALL))
                .highlight_style(Style::default().bg(Color::Green).modifier(Modifier::BOLD));
            f.render_stateful_widget(block, chunks[0], explorer.list_state());
        })?;

        player.advance_if_empty()?;

        // FIXME make it less panicky looking
        match io::stdin().events().next().unwrap().unwrap() {
            Event::Key(Key::Char('q')) => {
                // TODO: use termion alternate screen to preserve the terminal's state prior to
                //       opening this
                terminal.clear()?;
                return Ok(());
            },
            Event::Key(Key::Char('j')) => {
                explorer.select_next();
            },
            Event::Key(Key::Char('k')) => {
                explorer.select_previous();
            },
            Event::Key(Key::Char('h')) => {
                explorer.select_previous_dir();
            },
            Event::Key(Key::Char('l')) => {
                explorer.select_next_dir()?;
            },
            Event::Key(Key::Char('g')) => {
                explorer.top();
            },
            Event::Key(Key::Char('G')) => {
                explorer.bottom();
            },
            Event::Key(Key::Char('\n')) => {
                match explorer.state() {
                    State::Songs => {
                        player.play_file(explorer.selected_dir().selected().path())?;
                    },
                    State::Albums => {
                        player.play_album(explorer.selected_dir().selected().path())?;
                    },
                    _ => (),
                }
            },
            Event::Key(Key::Char('p')) => {
                player.toggle_pause()
            },
            _ => (),
        }
    }
}
