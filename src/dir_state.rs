use std::fs;
use std::io;
use std::path::{Path, PathBuf};

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
