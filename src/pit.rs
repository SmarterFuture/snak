use std::{collections::LinkedList, usize};
use crate::backend;
use rand::Rng;


#[repr(u8)]
pub enum Tile {
    Border = b'#',
    Ded = b'X',
    Empty = b' ',
    Munch = b'+',
    Snak = b'O',
}

pub enum Collision {
    No,
    Munch,
    Ded,
}


pub struct Pit {
    status_line: Vec<u8>,
    pub buf: Vec<u8>,
    size: backend::Size,
}

impl Pit {

    pub fn blit(&self) {
        backend::write(&[&self.status_line[..], &self.buf[..]].concat());
    }
    
    pub fn empty(&mut self) {
        let flat = self.size.flatten() - self.size.cols;
        self.buf = (0..flat).map(|x| match x {
            x if ( x < self.size.cols || x > flat - self.size.cols || 
                x % self.size.cols == 0 || (x + 1) % self.size.cols == 0 ) => Tile::Border,
            _ => Tile::Empty,
        } as u8).collect();
        self.set_status(0, 0);
    }

    pub fn get_size(&self) -> backend::Size {
        backend::Size {
            rows: self.size.rows - 1,
            cols: self.size.cols,
        }
    }

    pub fn is_collision(&self, pos: usize) -> Collision {
        match self.buf[pos] {
            x if x == Tile::Empty as u8 => Collision::No,
            x if x == Tile::Munch as u8 => Collision::Munch,
            _ => Collision::Ded,
        }
    }

    pub fn kill_snak(&mut self, body: LinkedList<usize>) {
        for i in body {
            self.buf[i] = Tile::Ded as u8;
        };
        let mid = self.size.middle();
        for (i, v) in b"u ded.".iter().enumerate() {
            self.buf[mid - 3 + i] = *v;
        };
        self.blit();
    }

    pub fn next_munch(&mut self, rng: &mut impl Rng){
        loop {
            let tmp: usize = rng.gen_range(0..self.size.flatten() - self.size.cols);
            if self.buf[tmp] == Tile::Empty as u8 &&
                tmp % self.size.cols % 2 == self.size.cols / 2 % 2 {
                    self.buf[tmp] = Tile::Munch as u8;
                    break;
            }
        };
    }

    pub fn new(size: backend::Size) -> Self {
        Pit {
            status_line: Vec::with_capacity(size.cols),
            buf: Vec::with_capacity(size.flatten()),
            size,
        }
    }

    pub fn set(&mut self, head: usize, tail: Option<usize>) {
        self.buf[head] = Tile::Snak as u8;
        if let Some(x) = tail {self.buf[x] = Tile::Empty as u8};
    }

    pub fn set_status(&mut self, score: u16, tick: u16) {
        self.status_line = format!("S {:0>6} | T {:06x}{:<len$}", score, tick, "",
            len=self.size.cols - 19).into_bytes();
    }

    pub fn st_screen(&mut self) {
        let mid = self.size.middle();
        for (i, v) in b"Snak".iter().enumerate() {
            self.buf[mid - 2 + i - self.size.cols] = *v;
        };
        for (i, v) in b"to cheap to E...".iter().enumerate() {
            self.buf[mid - 7 + i] = *v;
        };
        self.blit();
    }

}
