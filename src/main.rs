extern crate tui;
extern crate termion;

use std::io;
use std::fs;
use std::ffi::OsStr;
use std::fs::DirEntry;
use std::convert::AsRef;
use std::path::{Path, PathBuf};

use termion::raw::IntoRawMode;
use termion::event::{Event, Key};
use termion::input::TermRead;

use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::{List, ListState, Block, Borders, Text};
use tui::layout::{Layout, Constraint, Direction};
use tui::style::{Color, Style, Modifier};

enum State {
    Artists,
    Albums,
    Songs,
}

fn read_dir<P: AsRef<Path>, F>(path: P, check: F) -> io::Result<Vec<DirEntry>>
    where F: Fn(PathBuf) -> bool {
    Ok(fs::read_dir(path)?
        .filter(|de| check(de.as_ref().unwrap().path()))
        .map(|de| de.unwrap())
        .collect())
}

fn main() -> Result<(), io::Error> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;
    terminal.hide_cursor()?;

    let artist_paths = read_dir("/home/tucker/Music", |p| p.is_dir())?;
    let mut artist_albums: Vec<DirEntry> = Vec::new();
    let mut album_songs: Vec<DirEntry> = Vec::new();

    let mut list_state = ListState::default();
    list_state.select(Some(0));
    let mut list_max = artist_paths.len();

    let mut state = State::Artists;

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

            let dir_iter = match state {
                State::Artists => artist_paths.iter(),
                State::Albums => artist_albums.iter(),
                State::Songs => album_songs.iter(),
            };
            let dir_strings = dir_iter.map(|de| {
                de.path().into_os_string().into_string().unwrap()
            });
            let block = List::new(dir_strings.map(|de| Text::raw(de)))
                .block(Block::default().title("Artists").borders(Borders::ALL))
                .highlight_style(Style::default().bg(Color::Green).modifier(Modifier::BOLD))
                .highlight_symbol(">>");
            f.render_stateful_widget(block, chunks[0], &mut list_state);
        })?;

        // FIXME make it less panicky looking
        match io::stdin().events().next().unwrap().unwrap() {
            Event::Key(Key::Char('q')) => {
                // TODO: use termion alternate screen to preserve the terminal's state prior to
                //       opening this
                terminal.clear()?;
                return Ok(());
            },
            Event::Key(Key::Char('j')) => {
                let selected = list_state.selected().unwrap();
                if list_max == 1 || selected > list_max - 2 {
                    list_state.select(Some(0));
                } else {
                    list_state.select(Some(selected + 1));
                }
            },
            Event::Key(Key::Char('k')) => {
                let selected = list_state.selected().unwrap();
                if selected == 0 {
                    list_state.select(Some(list_max - 1));
                } else {
                    list_state.select(Some(selected - 1));
                }
            },
            Event::Key(Key::Char('h')) => {
                state = match state {
                    State::Albums => {
                        list_state.select(Some(0));
                        list_max = artist_paths.len();
                        State::Artists
                    },
                    State::Songs => {
                        list_state.select(Some(0));
                        list_max = artist_albums.len();
                        State::Albums
                    },
                    _ => state,
                };
            },
            Event::Key(Key::Char('l')) => {
                state = match state {
                    State::Artists => {
                        let selected = list_state.selected().unwrap();
                        artist_albums = read_dir(artist_paths[selected].path(), |p| p.is_dir())
                            .expect("error reading dir");
                        list_state.select(Some(0));
                        list_max = artist_albums.len();
                        State::Albums
                    },
                    State::Albums => {
                        let selected = list_state.selected().unwrap();
                        album_songs = read_dir(artist_albums[selected].path(), |p| {
                            match p.extension() {
                                Some(s) => {
                                    for &e in &[OsStr::new("wav"), OsStr::new("flac")] {
                                        if s == e {
                                            return true;
                                        }
                                    }
                                    false
                                },
                                None => false,
                            }
                        }).expect("error reading dir");
                        list_state.select(Some(0));
                        list_max = album_songs.len();
                        State::Songs
                    },
                    _ => state,
                };
            }
            _ => (),
        }
    }
}
