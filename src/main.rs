use std::io;

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, List, ListState, Text};
use tui::Terminal;

use bebop::*;

mod player;
use player::Player;

fn main() -> Result<(), io::Error> {
    let mut player = Player::new(0.2)
        // FIXME: handle it, dummy
        .expect("error creating player");
    let music_dir = std::env::var("BEBOP_MUSIC_DIR")
        .expect("BEBOP_MUSIC_DIR not set");
    let mut explorer = Explorer::new(music_dir)?;

    let mut stdin = io::stdin().keys();
    let stdout = io::stdout().into_raw_mode()?;
    let screen = termion::screen::AlternateScreen::from(stdout);
    let backend = TermionBackend::new(screen);
    let mut terminal = Terminal::new(backend)?;

    terminal.hide_cursor()?;

    let mut playing_selected = ListState::default();
    playing_selected.select(None);

    let mut search = String::new();

    loop {
        terminal.draw(|mut f| {
            let constraints = if search.is_empty() {
                [Constraint::Percentage(100), Constraint::Percentage(0)]
            } else {
                [Constraint::Percentage(98), Constraint::Percentage(2)]
            };
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints.as_ref())
                .split(f.size());
            let main = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(chunks[0]);

            let dir_strings = explorer.selected_dir().entry_strings();
            let current_dir = explorer
                .current_dir_name()
                .unwrap_or_else(|| "Music".to_string());
            let block = List::new(dir_strings.iter().map(Text::raw))
                .block(Block::default().title(&current_dir).borders(Borders::ALL))
                .highlight_style(Style::default().bg(Color::Green).modifier(Modifier::BOLD));
            f.render_stateful_widget(block, main[0], explorer.list_state());

            if !player.playing().is_empty() {
                playing_selected.select(Some(player.index()));
            }
            let playing_strings: Vec<String> = player
                .playing()
                .iter()
                .map(|p| p.file_name().unwrap().to_os_string().into_string().unwrap())
                .collect();
            let volume = format!("Volume: {:.0}", player.volume() * 100f32);
            let block = List::new(playing_strings.iter().map(Text::raw))
                .block(Block::default().title(&volume).borders(Borders::ALL))
                .highlight_style(Style::default().bg(Color::Green).modifier(Modifier::BOLD));
            f.render_stateful_widget(block, main[1], &mut playing_selected);

            let search_bar = Block::default()
                .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                .title(&search);
            f.render_widget(search_bar, chunks[1]);
        })?;

        if let Some(s) = stdin.next() {
            if let Ok(key) = s {
                if !search.is_empty() {
                    if let Key::Char(c) = key {
                        if c == '\n' {
                            search.clear();
                            continue;
                        }
                        search.push(c);
                        explorer.search(&search[1..]);
                    } else if let Key::Backspace = key {
                        search.pop();
                    } else {
                        search.clear();
                    }
                    // kinda dumb
                    continue;
                }
            }

            match s {
                Ok(k) => match k {
                    Key::Char('q') => break,
                    Key::Char('j') => {
                        explorer.select_next();
                    }
                    Key::Char('k') => {
                        explorer.select_previous();
                    }
                    Key::Char('h') => {
                        explorer.select_previous_dir();
                    }
                    Key::Char('l') => {
                        explorer.select_next_dir()?;
                    }
                    Key::Char('g') => {
                        explorer.top();
                    }
                    Key::Char('G') => {
                        explorer.bottom();
                    }
                    Key::Char('\n') => match explorer.state() {
                        State::Songs => {
                            player.play_song(explorer.selected().clone())?;
                        }
                        State::Albums => {
                            explorer.select_next_dir()?;
                            player.play_songs(0, explorer.selected_dir().dir().clone())?;
                            explorer.select_previous_dir();
                        }
                        _ => (),
                    },
                    Key::Char('p') => player.toggle_pause(),
                    Key::Char('-') => {
                        let volume = player.volume() - 0.01f32;
                        player.set_volume(volume);
                    }
                    Key::Char('+') => {
                        let volume = player.volume() + 0.01f32;
                        player.set_volume(volume);
                    }
                    Key::Char('b') => {
                        let index = player.index();
                        if index > 0 {
                            player.play_songs(index - 1, player.playing().to_vec())?;
                        }
                    }
                    Key::Char('w') => {
                        let index = player.index();
                        if index < player.playing().len() - 1 {
                            player.play_songs(index + 1, player.playing().to_vec())?;
                        }
                    }
                    Key::Char('/') => {
                        search.push('/');
                    }
                    _ => (),
                },
                Err(e) => eprintln!("{}", e),
            }
        }
    }

    Ok(())
}
