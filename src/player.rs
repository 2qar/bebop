extern crate rodio;

use std::io;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

pub struct Player {
    _stream: rodio::OutputStream,
    stream_handle: rodio::OutputStreamHandle,
    sink: rodio::Sink,
    volume: f32,
}

impl Player {
    pub fn new(volume: f32) -> Result<Player, rodio::StreamError> {
        let (_stream, stream_handle) = rodio::OutputStream::try_default()?;
        let (sink, _) = rodio::Sink::new_idle();

        Ok(Player { _stream, stream_handle, sink, volume })
    }

    fn reset_sink(&mut self) {
        // FIXME: actually handle the error instead of just expecting
        self.sink = rodio::Sink::try_new(&self.stream_handle)
            .expect("error opening sink");
        self.sink.set_volume(self.volume);
    }

    pub fn play_file(&mut self, p: PathBuf) -> io::Result<()> {
        let f = File::open(p)?;
        let source = rodio::Decoder::new(BufReader::new(f))
            // FIXME: handle the error, dummy
            .expect("error decoding file");

        self.reset_sink();
        self.sink.append(source);

        Ok(())
    }

    pub fn play_album(&mut self, dir: &Vec<PathBuf>) -> io::Result<()> {
        self.reset_sink();

        for path in dir {
            let f = File::open(path)?;
            let source = rodio::Decoder::new(BufReader::new(f))
                .expect("error decoding file");
            self.sink.append(source);
        }

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

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn set_volume(&mut self, v: f32) {
        self.volume = if v < 0f32 {
            0f32
        } else if v > 1f32 {
            1f32
        } else {
            v
        };
        self.sink.set_volume(self.volume);
    }
}
