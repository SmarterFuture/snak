use std::collections::LinkedList;
use crate::backend;


#[derive(Debug)]
pub enum Dir {
    UP,
    DOWN,
    LEFT,
    RIGHT
}

impl Dir {
    pub fn is_vertical(&self) -> bool {
        match &self {
            Dir::UP | Dir::DOWN => true,
            Dir::LEFT | Dir::RIGHT => false,
        }
    }
}

#[derive(Debug)]
pub struct Snak {
    dir: Dir,
    _body: LinkedList<usize>,
    _pit: backend::Size,
}

impl Snak {
    pub fn change_dir(&mut self, new: Dir) {
        if new.is_vertical() ^ self.dir.is_vertical() {
            self.dir = new;
        };
    }

    pub fn ded(self) -> LinkedList<usize> {
        let snak = self;
        snak._body
    }

    pub fn munch(&mut self) -> usize {
        let head: usize = *self._body.front().unwrap_or(&self._pit.middle());
        let max = self._pit.flatten();
        let row: usize = head / self._pit.cols * self._pit.cols;
        self._body.push_front(
            match self.dir {
                Dir::UP => (head + max - self._pit.cols) % max,
                Dir::DOWN => (head + max + self._pit.cols) % max,
                Dir::LEFT => row + (head + self._pit.cols - 2) % self._pit.cols,
                Dir::RIGHT => row + (head + 2) % self._pit.cols,
            });
        *self._body.front().unwrap()
    }

    pub fn new(pit: backend::Size) -> Self {
        Snak {
            dir: Dir::UP,
            _body: LinkedList::from([pit.middle()]),
            _pit: pit
        }
    }

    pub fn tailoff(&mut self) -> usize {
        self._body.pop_back().unwrap()
    }
}
