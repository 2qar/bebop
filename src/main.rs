use std::io;
use std::io::Write;
use std::sync::mpsc::channel;
use std::thread;
use std::fs::OpenOptions;
use std::path::PathBuf;

use signal_hook::iterator::Signals;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::Terminal;

use bebop::input::{Action, handle_input, send_input};
use bebop::layout::draw;
use bebop::{Event, Explorer, Player};

fn main() -> Result<(), io::Error> {
    let mut player = Player::new(0.2).expect("error creating player");
    let music_dir = std::env::var("BEBOP_MUSIC_DIR").expect("BEBOP_MUSIC_DIR not set");
    let mut explorer = Explorer::new(music_dir)?;
    let status_file_path = std::env::var("BEBOP_STATUS_FILE_PATH").unwrap_or_default();

    let stdout = io::stdout().into_raw_mode()?;
    let screen = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(screen);
    let mut terminal = Terminal::new(backend)?;

    terminal.hide_cursor()?;

    let mut search = String::new();
    let (event_sender, event_receiver) = channel::<Event>();

    let input_sender = event_sender.clone();
    thread::spawn(move || {
        send_input(input_sender);
    });

    let resize_sender = event_sender.clone();
    let signals = Signals::new(&[signal_hook::SIGWINCH])?;
    thread::spawn(move || {
        for _ in signals.forever() {
            if let Err(e) = resize_sender.send(Event::Redraw) {
                eprintln!("error writing to event channel: {}", e);
            }
        }
    });

    loop {
        //FIXME: this is really long and bad and gross.
        //     ewwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwww
        draw::<TermionBackend<AlternateScreen<RawTerminal<io::Stdout>>>>(
            &mut terminal,
            &mut explorer,
            &mut player,
            &search,
        )?;

        match event_receiver.recv() {
            Ok(event) => match handle_input(
                    event,
                    &mut explorer,
                    &mut player,
                    &mut search,
                ) {
                Ok(a) => match a {
                    Action::Play(song_switch_receiver) => {
                        let songs = player.playing().clone();

                        let path = if status_file_path.is_empty() {
                            String::new()
                        } else {
                            status_file_path.to_owned()
                        };
                        if let Err(e) = write_status(&path, &songs[player.index()]) {
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
                    Action::Quit => break,
                    Action::None => (),
                }
                Err(e) => eprintln!("error handling input: {}", e),
            }
            Err(e) => println!("error receiving event: {}", e),
        }
    }

    Ok(())
}

// TODO: move this to a new file along with the song switch stuff, maybe
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
