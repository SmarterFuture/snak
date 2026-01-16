use rand::Rng;

use crate::backend::{RenderTarget, Renderble, Size};
use std::collections::LinkedList;

#[derive(Debug)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Tile {
    Corpse,
    Empty,
    Food,
    Snake,
}

#[derive(Debug)]
pub struct Status {
    pub score: usize,
    pub diff: usize,
}

impl Default for Status {
    fn default() -> Self {
        Self { score: 1, diff: 0 }
    }
}

impl Dir {
    pub fn is_vertical(&self) -> bool {
        match &self {
            Dir::Up | Dir::Down => true,
            Dir::Left | Dir::Right => false,
        }
    }
}

#[derive(Debug)]
pub struct Snake<R>
where
    R: Rng,
{
    dir: Dir,
    body: LinkedList<usize>,
    bbox: Size,
    buf: Vec<Tile>,
    rng: R,
    status: Status,
}

impl<R> Snake<R>
where
    R: Rng,
{
    pub fn new(bbox: Size, rng: R) -> Self {
        let middle = bbox.middle();
        let mut buf: Vec<_> = (0..bbox.flatten()).map(|_| Tile::Empty).collect();
        buf[middle] = Tile::Snake;

        Snake {
            dir: Dir::Up,
            body: LinkedList::from([middle]),
            bbox,
            buf,
            rng,
            status: Status::default(),
        }
    }

    pub fn change_dir(&mut self, new: Dir) {
        if new.is_vertical() ^ self.dir.is_vertical() {
            self.dir = new;
        };
    }

    fn advance(&mut self, idx: usize) {
        self.body.push_front(idx);
        self.buf[idx] = Tile::Snake;
    }

    fn tailoff(&mut self) {
        let idx = self.body.pop_back().unwrap();
        self.buf[idx] = Tile::Empty
    }

    fn next_food(&mut self) {
        loop {
            let food = self.rng.gen_range(0..self.bbox.flatten() - self.bbox.w);
            if self.buf[food] == Tile::Empty {
                self.buf[food] = Tile::Food;
                break;
            }
        }
    }

    pub fn start(&mut self) {
        self.next_food();
    }

    fn death(&mut self) {
        for i in &self.body {
            self.buf[*i] = Tile::Corpse;
        }
    }

    fn change_score(&mut self) {
        self.status.score += 1;
        self.status.diff += (self.status.score % 10 == 0) as usize
    }

    /// returns whether the snake died or not
    pub fn tick_move(&mut self) -> Result<usize, ()> {
        let head: usize = *self.body.front().unwrap_or(&self.bbox.middle());
        let x = head % self.bbox.w;
        let y = head / self.bbox.w;

        let future_head = match self.dir {
            Dir::Up => {
                if y == 0 {
                    None
                } else {
                    Some(head - self.bbox.w)
                }
            }
            Dir::Down => {
                if (y + 1) >= self.bbox.h {
                    None
                } else {
                    Some(head + self.bbox.w)
                }
            }
            Dir::Left => {
                if x == 0 {
                    None
                } else {
                    Some(head - 1)
                }
            }
            Dir::Right => {
                if (x + 1) >= self.bbox.w {
                    None
                } else {
                    Some(head + 1)
                }
            }
        };

        if let Some(idx) = future_head {
            match self.buf[idx] {
                Tile::Food => {
                    self.advance(idx);
                    self.next_food();
                    self.change_score();
                    Ok(self.status.diff)
                }
                Tile::Empty => {
                    self.advance(idx);
                    self.tailoff();
                    Ok(self.status.diff)
                }
                _ => {
                    self.death();
                    Err(())
                }
            }
        } else {
            self.death();
            Err(())
        }
    }

    pub fn draw_snake_to<F>(&self, target: &mut F) -> Result<(), F::Error>
    where
        F: RenderTarget<<Tile as Renderble>::Primitive>,
    {
        target.draw(self.buf.iter().flat_map(|x| x.render()))
    }

    pub fn draw_status_to<F>(&self, target: &mut F) -> Result<(), F::Error>
    where
        F: RenderTarget<<Status as Renderble>::Primitive>,
    {
        target.draw(self.status.render())
    }
}
