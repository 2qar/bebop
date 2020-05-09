use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use rodio::source::Done;

pub struct Player {
    _stream: rodio::OutputStream,
    stream_handle: rodio::OutputStreamHandle,
    sink: rodio::Sink,
    volume: f32,
    playing: Vec<PathBuf>,
    remaining: Arc<AtomicUsize>,
}

impl Player {
    pub fn new(volume: f32) -> Result<Player, rodio::StreamError> {
        let (_stream, stream_handle) = rodio::OutputStream::try_default()?;
        let (sink, _) = rodio::Sink::new_idle();

        let playing = Vec::new();
        let remaining = Arc::new(AtomicUsize::new(0));

        Ok(Player {
            _stream,
            stream_handle,
            sink,
            volume,
            playing,
            remaining,
        })
    }

    fn reset_sink(&mut self) {
        // FIXME: actually handle the error instead of just expecting
        self.sink = rodio::Sink::try_new(&self.stream_handle).expect("error opening sink");
        self.sink.set_volume(self.volume);
    }

    pub fn play_song(&mut self, p: PathBuf) -> io::Result<()> {
        self.play_songs(0, vec![p])
    }

    pub fn play_songs(&mut self, start: usize, dir: Vec<PathBuf>) -> io::Result<()> {
        self.reset_sink();
        let remaining = &self.remaining;
        remaining.store(dir.len() - start, Ordering::Relaxed);
        self.playing = dir.clone();

        for path in dir[start..].to_vec() {
            let f = File::open(path)?;
            let source = rodio::Decoder::new(BufReader::new(f)).expect("error decoding file");
            self.sink.append(Done::new(source, self.remaining.clone()));
        }

        Ok(())
    }

    pub fn playing(&self) -> &Vec<PathBuf> {
        &self.playing
    }

    pub fn index(&self) -> usize {
        self.playing.len() - self.remaining.load(Ordering::Relaxed)
    }

    pub fn toggle_pause(&self) {
        if self.sink.is_paused() {
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
