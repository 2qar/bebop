extern crate tui;
extern crate termion;
extern crate rodio;

use std::io;
use std::fs;

use termion::raw::IntoRawMode;
use termion::event::{Event, Key};
use termion::input::TermRead;

use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::{List, Block, Borders, Text};
use tui::layout::{Layout, Constraint, Direction};
use tui::style::{Color, Style, Modifier};

use bebop::*;

fn main() -> Result<(), io::Error> {
    let device = rodio::default_output_device().expect("error opening audio device");
    let mut sink = rodio::Sink::new(&device);
    sink.set_volume(0.2);

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
                        sink = rodio::Sink::new(&device);
                        sink.set_volume(0.2);

                        let f = fs::File::open(explorer.selected().path())?;
                        let source = rodio::Decoder::new(io::BufReader::new(f)).expect("error decoding file");
                        sink.append(source);
                    },
                    _ => (),
                }
            },
            Event::Key(Key::Char('p')) => {
                if sink.is_paused() {
                    sink.play();
                } else {
                    sink.pause();
                }
            },
            _ => (),
        }
    }
}
