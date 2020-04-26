extern crate rodio;

use std::io;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use rodio::{Sink, Device};

pub struct Player {
    device: Device,
    sink: Sink,
    volume: f32,
    sources: Vec<PathBuf>,
}

impl Player {
    pub fn new(volume: f32) -> Player {
        let device = rodio::default_output_device().expect("error opening audio device");
        let sink = Sink::new(&device);
        sink.set_volume(volume);

        Player { device, sink, volume, sources: Vec::new() }
    }

    pub fn play_file(&mut self, p: PathBuf) -> io::Result<()> {
        self.sink = Sink::new(&self.device);
        self.sink.set_volume(self.volume);

        let f = File::open(p)?;
        let source = rodio::Decoder::new(BufReader::new(f)).expect("error decoding file");
        self.sink.append(source);

        Ok(())
    }

    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }

    pub fn toggle_pause(&self) {
        if self.is_paused() {
            self.sink.play()
        } else {
            self.sink.pause()
        }
    }
}
