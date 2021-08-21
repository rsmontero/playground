use std::io::{stdin, BufRead, BufReader};

use std::{
    fs::File,
    os::unix::io::{AsRawFd, FromRawFd, RawFd},
};

pub trait LineReader {
    fn read_line(&mut self) -> Option<String>;
}

pub struct LineBuffer<BR: BufRead> {
    reader: BR,
}

impl LineBuffer<BufReader<File>> {
    pub fn from_path(path: &str) -> Option<Self> {
        let f = match File::open(path) {
            Ok(f) => f,
            Err(_) => return None,
        };

        Some(LineBuffer {
            reader: std::io::BufReader::new(f),
        })
    }

    pub fn from_fd(_fd: RawFd) -> LineBuffer<BufReader<File>> {
        let f = unsafe { File::from_raw_fd(_fd) };

        LineBuffer {
            reader: std::io::BufReader::new(f),
        }
    }

    pub fn from_stdin() -> LineBuffer<BufReader<File>> {
        LineBuffer::from_fd(stdin().as_raw_fd())
    }
}

impl<T: BufRead> LineReader for LineBuffer<T> {
    fn read_line(&mut self) -> Option<String> {
        let mut s = String::new();

        match self.reader.read_line(&mut s) {
            Ok(l) => {
                if l == 0 {
                    // TODO deal EOF Result<...> or trait fn
                    None
                } else {
                    Some(s)
                }
            }
            Err(_) => None,
        }
    }
}
