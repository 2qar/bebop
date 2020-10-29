use std::io;
use std::sync::mpsc::channel;
use std::thread;

use signal_hook::iterator::Signals;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::Terminal;

use bebop::input::{handle_input, send_input};
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

    // TODO: give an event_sender to the player for when songs change

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
            Ok(event) => {
                // FIXME: 6 parameters? gotta be a better way
                let quit = handle_input(
                    event,
                    &mut explorer,
                    &mut player,
                    &mut search,
                    &status_file_path,
                    &event_sender,
                )?;
                if quit {
                    break;
                }
            }
            Err(e) => println!("error receiving event: {}", e),
        }
    }

    Ok(())
}
