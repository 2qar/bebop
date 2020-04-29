extern crate rodio;

use std::fs;

use std::io;
use std::fs::File;
use std::io::{BufReader, ErrorKind};
use std::path::PathBuf;

pub struct Player {
    stream: rodio::OutputStream,
    stream_handle: rodio::OutputStreamHandle,
    sink: rodio::Sink,
    volume: f32,
    playlist: Playlist,
}

impl Player {
    pub fn new(volume: f32) -> Result<Player, rodio::StreamError> {
        let (stream, stream_handle) = rodio::OutputStream::try_default()?;
        let (sink, _) = rodio::Sink::new_idle();

        Ok(Player { stream, stream_handle, sink, volume, playlist: Playlist::new(Vec::new()) })
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

    pub fn play_album(&mut self, p: PathBuf) -> io::Result<()> {
        if !p.is_dir() {
            return Err(io::Error::new(ErrorKind::InvalidInput, "path isn't a directory"));
        }

        let mut song_paths: Vec<PathBuf> = Vec::new();
        for e in fs::read_dir(p)? {
            song_paths.push(e.unwrap().path());
        }

        self.reset_sink();
        self.playlist = Playlist::new(song_paths);
        Ok(())
    }

    /// advance_if_empty starts playing the next song if nothing is playing
    pub fn advance_if_empty(&mut self) -> io::Result<()> {
        if self.sink.empty() {
            match self.playlist.next() {
                Some(p) => {
                    self.play_file(p)?;
                },
                None => (),
            };
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
}

struct Playlist {
    paths: Vec<PathBuf>,
}

impl Playlist {
    pub fn new(paths: Vec<PathBuf>) -> Playlist {
        Playlist { paths }
    }
}

impl Iterator for Playlist {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        if self.paths.len() > 0 {
            Some(self.paths.remove(0))
        } else {
            None
        }
    }
}
