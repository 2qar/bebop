use crate::{Event, Explorer, Player, State};
use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::thread;
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

pub fn handle_input(
    event: Event,
    explorer: &mut Explorer,
    player: &mut Player,
    search: &mut String,
    status_file_path: &str,
    event_sender: &Sender<Event>,
) -> io::Result<bool> {
    let key = match event {
        Event::Input(k) => k,
        _ => return Ok(false),
    };
    if !search.is_empty() {
        if let Key::Char(c) = key {
            if c == '\n' {
                search.clear();
                return Ok(false);
            }
            search.push(c);
            explorer.search(&search[1..]);
        } else if let Key::Backspace = key {
            search.pop();
        } else {
            search.clear();
        }

        return Ok(false);
    }

    let mut quit = false;
    match key {
        Key::Char('q') => {
            quit = true;
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
                let songs = explorer.selected_dir().dir().clone();
                explorer.select_previous_dir();

                let path = if status_file_path.is_empty() {
                    String::new()
                } else {
                    status_file_path.to_owned()
                };
                if let Err(e) = write_status(&path, &songs[0]) {
                    eprintln!("error writing status: {}", e);
                }
                let redraw_sender = event_sender.clone();
                thread::spawn(move || {
                    while let Ok(i) = song_switch_receiver.recv() {
                        if let Err(e) = redraw_sender.send(Event::Redraw) {
                            eprintln!("error sending redraw on song change: {}", e);
                        }
                        if i != 0 {
                            if let Err(e) = write_status(&path, &songs[songs.len() - i]) {
                                eprintln!("error writing status: {}", e);
                            }
                        }
                    }
                });
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
    }

    Ok(quit)
}

fn write_status(path: &str, playing: &PathBuf) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)?;
    // FIXME: these are fuckin dumb
    // TODO: strip ".mp3" and maybe track # from filename
    let song_name = playing.file_name().unwrap().to_str().unwrap();
    let artist = playing
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();

    write!(
        &mut file,
        "{}\n{}\n{}/cover.jpg\n",
        song_name,
        artist,
        playing.parent().unwrap().to_str().unwrap()
    )
}
