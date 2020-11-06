use crate::{Event, Explorer, Player, State};
use std::io;
use std::sync::mpsc::{Sender, Receiver};
use termion::event::Key;
use termion::input::TermRead;

pub fn send_input(s: Sender<Event>) {
    let mut stdin = io::stdin().keys();

    loop {
        if let Some(i) = stdin.next() {
            match i {
                Ok(key) => s.send(Event::Input(key)).unwrap(),
                Err(e) => eprintln!("error reading key: {}", e),
            }
        }
    }
}

pub enum Action {
    Play(Receiver<usize>),
    Quit,
    None,
}

pub fn handle_input(
    event: Event,
    explorer: &mut Explorer,
    player: &mut Player,
    search: &mut String,
) -> io::Result<Action> {
    let key = match event {
        Event::Input(k) => k,
        _ => return Ok(Action::None),
    };
    if !search.is_empty() {
        if let Key::Char(c) = key {
            if c == '\n' {
                search.clear();
                return Ok(Action::None);
            }
            search.push(c);
            explorer.search(&search[1..]);
        } else if let Key::Backspace = key {
            search.pop();
        } else {
            search.clear();
        }

        return Ok(Action::None);
    }

    let mut action = Action::None;
    match key {
        Key::Char('q') => {
            action = Action::Quit;
        }
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
                let song_switch_receiver =
                    player.play_songs(0, explorer.selected_dir().dir().clone())?;
                explorer.select_previous_dir();
                action = Action::Play(song_switch_receiver);
            }
            State::Artists => {
                explorer.select_next_dir()?;
            }
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
                action = Action::Play(player.play_songs(index - 1, player.playing().to_vec())?);
            }
        }
        Key::Char('w') => {
            let index = player.index();
            if index < player.playing().len() - 1 {
                action = Action::Play(player.play_songs(index + 1, player.playing().to_vec())?);
            }
        }
        Key::Char('/') => {
            search.push('/');
        }
        _ => (),
    }

    Ok(action)
}
