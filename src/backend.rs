use std::io::{stdout, Write};
use std::usize;
use libc::{STDOUT_FILENO, TIOCGWINSZ, c_ushort};
use libc::ioctl;

#[repr(C)]
#[derive(Debug)]
struct UnixSize {
    rows: c_ushort,
    cols: c_ushort,
}

#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub rows: usize,
    pub cols: usize,
}

impl Size {
    pub fn middle(&self) -> usize {
        self.rows / 2 * self.cols + self.cols / 2
    }

    pub fn flatten(&self) -> usize {
        self.rows * self.cols
    }
}

pub fn write(buf: &[u8]) {
    let stdout = stdout();
    let mut lock = stdout.lock();
    lock.write("\r\x1b[2J\r\x1b[0;0H\r\x1b[?25l".as_bytes()).unwrap();
    lock.write(buf).unwrap();
    lock.flush().unwrap();
    drop(lock);
}

pub fn get_size() -> Option<Size> {
    let us = UnixSize {
        rows: 0,
        cols: 0
    };

    let tmp = unsafe { ioctl(STDOUT_FILENO, TIOCGWINSZ.into(), &us) };
    if tmp == 0 {
        Some( Size {
            rows: us.rows as usize,
            cols: us.cols as usize,
        })
    } else {
        None
    }
}
