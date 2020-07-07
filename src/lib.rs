use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use tui::widgets::ListState;

pub type Dir = Vec<PathBuf>;

#[derive(Default)]
pub struct DirState {
    index: usize,
    dir: Dir,
}

impl DirState {
    pub fn read_dir<P: AsRef<Path>, F>(path: P, check: F) -> io::Result<DirState>
    where
        F: Fn(PathBuf) -> bool,
    {
        let mut dir = read_dir(path, check)?;
        dir.sort();

        Ok(DirState { index: 0, dir })
    }

    pub fn entry_strings(&self) -> Vec<String> {
        self.dir
            .iter()
            .map(|p| p.file_name().unwrap().to_os_string().into_string().unwrap())
            .collect()
    }

    pub fn dir(&self) -> &Dir {
        &self.dir
    }

    pub fn entries(&self) -> usize {
        self.dir.len()
    }

    pub fn select_next(&mut self) -> usize {
        if self.index == self.dir.len() - 1 {
            self.index = 0;
        } else {
            self.index += 1;
        }
        self.index
    }

    pub fn select_previous(&mut self) -> usize {
        if self.index == 0 {
            self.index = self.dir.len() - 1;
        } else {
            self.index -= 1;
        }
        self.index
    }

    pub fn select(&mut self, i: usize) -> Option<usize> {
        if i > self.dir.len() - 1 {
            None
        } else {
            self.index = i;
            Some(i)
        }
    }

    pub fn selected(&self) -> &PathBuf {
        &self.dir[self.index]
    }

    pub fn selected_name(&self) -> Option<String> {
        match self.selected().file_name() {
            Some(s) => match s.to_os_string().into_string() {
                Ok(s) => Some(s),
                Err(_) => None,
            },
            None => None,
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn find(&self, name: &str) -> Option<usize> {
        self.entry_strings().iter().position(|s| {
            if let Some(_) = s.to_lowercase().find(&name.to_lowercase()) {
                return true;
            }
            return false;
        })
    }
}

pub fn read_dir<P: AsRef<Path>, F>(path: P, check: F) -> io::Result<Dir>
where
    F: Fn(PathBuf) -> bool,
{
    Ok(fs::read_dir(path)?
        .filter(|de| check(de.as_ref().unwrap().path()))
        .map(|de| de.unwrap().path())
        .collect())
}

#[derive(Copy, Clone)]
pub enum State {
    Artists,
    Albums,
    Songs,
}

pub struct Explorer {
    dirs: [DirState; 3],
    state: State,
    list_state: ListState,
}

impl Explorer {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Explorer> {
        let dirs = [
            DirState::read_dir(path, |p| p.is_dir())?,
            DirState::default(),
            DirState::default(),
        ];
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Ok(Explorer {
            dirs,
            state: State::Artists,
            list_state,
        })
    }

    pub fn selected_dir(&self) -> &DirState {
        match self.state {
            State::Artists => &self.dirs[0],
            State::Albums => &self.dirs[1],
            State::Songs => &self.dirs[2],
        }
    }

    fn selected_dir_mut(&mut self) -> &mut DirState {
        match self.state {
            State::Artists => &mut self.dirs[0],
            State::Albums => &mut self.dirs[1],
            State::Songs => &mut self.dirs[2],
        }
    }

    pub fn selected(&self) -> &PathBuf {
        self.selected_dir().selected()
    }

    pub fn select_next(&mut self) {
        let index = self.selected_dir_mut().select_next();
        self.list_state.select(Some(index));
    }

    pub fn select_previous(&mut self) {
        let index = self.selected_dir_mut().select_previous();
        self.list_state.select(Some(index));
    }

    pub fn select_next_dir(&mut self) -> io::Result<()> {
        match self.state {
            State::Artists => {
                self.dirs[1] = DirState::read_dir(self.dirs[0].selected(), |p| p.is_dir())?;
                self.state = State::Albums;
            }
            State::Albums => {
                self.dirs[2] =
                    DirState::read_dir(self.dirs[1].selected(), |p| match p.extension() {
                        Some(s) => is_song(s),
                        None => false,
                    })?;
                self.state = State::Songs;
            }
            _ => (),
        }

        self.update_selection();

        Ok(())
    }

    pub fn select_previous_dir(&mut self) {
        match self.state {
            State::Albums => {
                self.state = State::Artists;
            }
            State::Songs => {
                self.state = State::Albums;
            }
            _ => (),
        }

        self.update_selection()
    }

    pub fn current_dir_name(&self) -> Option<String> {
        match self.state {
            State::Artists => Some("Music".to_string()),
            State::Albums => self.dirs[0].selected_name(),
            State::Songs => self.dirs[1].selected_name(),
        }
    }

    pub fn update_selection(&mut self) {
        let index = self.selected_dir().index();
        self.list_state.select(Some(index));
    }

    pub fn top(&mut self) {
        let index = self.selected_dir_mut().select(0);
        self.list_state.select(index);
    }

    pub fn bottom(&mut self) {
        let selected = self.selected_dir_mut();
        let len = selected.entries();
        if len > 0 {
            let index = selected.select(len - 1);
            self.list_state.select(index);
        }
    }

    pub fn search(&mut self, s: &str) {
        if let Some(i) = self.selected_dir().find(s) {
            self.selected_dir_mut().select(i);
            self.list_state.select(Some(i));
        }
    }

    pub fn state(&self) -> State {
        self.state
    }

    pub fn list_state(&mut self) -> &mut ListState {
        &mut self.list_state
    }
}

fn is_song(s: &OsStr) -> bool {
    for &e in &[OsStr::new("wav"), OsStr::new("flac"), OsStr::new("mp3")] {
        if s == e {
            return true;
        }
    }

    false
}
