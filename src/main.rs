use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::thread;

use termion::screen::AlternateScreen;
use termion::raw::{IntoRawMode, RawTerminal};

use tui::backend::TermionBackend;
use tui::Terminal;

use bebop::{Explorer, Player, Event};
use bebop::input::{send_input, handle_input};
use bebop::layout::draw;

fn main() -> Result<(), io::Error> {
    let mut player = Player::new(0.2).expect("error creating player");
    let music_dir = std::env::var("BEBOP_MUSIC_DIR").expect("BEBOP_MUSIC_DIR not set");
    let mut explorer = Explorer::new(music_dir)?;
    // TODO: reintegrate writing to status file w/ the new structure and stuff
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

    // TODO: give an event_sender to the player for when songs change
    // TODO: add a signal handler for SIGWINCH and give it an event_sender

    loop {
        //FIXME: this is really long and bad and gross.
        //     ewwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwww
        draw::<TermionBackend<AlternateScreen<RawTerminal<io::Stdout>>>>(&mut terminal, &mut explorer, &mut player, &search)?;
        
        match event_receiver.recv() {
            Ok(event) => {
                let quit = handle_input(event, &mut explorer, &mut player, &mut search)?;
                if quit {
                    break;
                }
            },
            Err(e) => println!("error receiving event: {}", e),
        }
    }

    Ok(())
}

// TODO: move this outta the main file
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
