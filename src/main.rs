extern crate tui;
extern crate termion;

use std::io;

use termion::raw::IntoRawMode;
use termion::event::Key;
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
    let mut player = Player::new(0.2)
        // FIXME: handle it, dummy
        .expect("error creating player");

    let mut stdin = io::stdin().keys();

    let stdout = io::stdout().into_raw_mode()?;
    let screen = termion::screen::AlternateScreen::from(stdout);
    let backend = TermionBackend::new(screen);
    let mut terminal = Terminal::new(backend)?;

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
            let volume = format!("Volume: {:.0}", player.volume() * 100f32);
            let block = List::new(dir_strings.iter().map(|de| Text::raw(de)))
                .block(Block::default().title(volume.as_str()).borders(Borders::ALL))
                .highlight_style(Style::default().bg(Color::Green).modifier(Modifier::BOLD));
            f.render_stateful_widget(block, chunks[0], explorer.list_state());

        })?;

        match stdin.next() {
            Some(s) => match s {
                Ok(k) => match k {
                    Key::Char('q') => break,
                    Key::Char('j') => {
                        explorer.select_next();
                    },
                    Key::Char('k') => {
                        explorer.select_previous();
                    },
                    Key::Char('h') => {
                        explorer.select_previous_dir();
                    },
                    Key::Char('l') => {
                        explorer.select_next_dir()?;
                    },
                    Key::Char('g') => {
                        explorer.top();
                    },
                    Key::Char('G') => {
                        explorer.bottom();
                    },
                    Key::Char('\n') => {
                        match explorer.state() {
                            State::Songs => {
                                player.play_file(explorer.selected().clone())?;
                            },
                            State::Albums => {
                                explorer.select_next_dir()?;
                                player.play_album(explorer.selected_dir().dir())?;
                                explorer.select_previous_dir();
                            },
                            _ => (),
                        }
                    },
                    Key::Char('p') => {
                        player.toggle_pause()
                    },
                    Key::Char('-') => {
                        let volume = player.volume() - 0.01f32;
                        player.set_volume(volume);
                    },
                    Key::Char('+') => {
                        let volume = player.volume() + 0.01f32;
                        player.set_volume(volume);
                    }
                    _ => (),
                },
                Err(e) => eprintln!("{}", e),
            },
            None => (),
        }
    }

    Ok(())
}
