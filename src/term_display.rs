use std::{
    io::{self, Write},
    iter,
};

use crate::{
    backend::{Quad, RenderTarget, Renderble, Size},
    snake::{Status, Tile},
};

impl Renderble for Tile {
    type Primitive = char;
    fn render(&self) -> impl Iterator<Item = Self::Primitive> {
        iter::once(match self {
            Tile::Corpse => 'x',
            Tile::Empty => ' ',
            Tile::Food => '+',
            Tile::Snake => 'o',
        })
    }
}

pub struct TermScreen {
    full_size: Size,
    constent_size: Size,
}

impl TermScreen {
    const BORDER: char = '#';
    const BORDER_WIDTH: usize = 1;

    pub fn new(w: usize, h: usize) -> Self {
        Self {
            full_size: Size::new((w + 1) * 2, h + 3),
            constent_size: Size::new(w, h),
        }
    }

    pub fn from_size(mut s: Size) -> Self {
        let cs = s;
        s.w = (s.w + 1) * 2;
        s.h += 3;
        Self {
            full_size: s,
            constent_size: cs,
        }
    }

    pub fn render_text(
        &self,
        x: usize,
        y: usize,
        text: String,
        quad: Quad,
    ) -> Result<(), io::Error> {
        let offset = match quad {
            Quad::Left => 0,
            Quad::Center => text.len() / 2,
            Quad::Right => text.len(),
        };
        let r_x = (x * 2).saturating_sub(offset).min(self.full_size.w);
        let r_y = (y + 1).min(self.full_size.h);
        let len = (self.full_size.w - r_x).min(text.len());

        print!("\x1B[H");
        print!("{}", "\n".repeat(r_y));
        print!("\x1B[{}C", r_x);
        print!(" {}", text.chars().take(len).collect::<String>());
        io::stdout().flush()
    }
}

impl RenderTarget<char> for TermScreen {
    type Error = io::Error;

    fn init(&self) -> Result<(), Self::Error> {
        print!("\x1B[?1049h");
        print!("\x1B[?25l");
        print!("\x1B[2J\x1B[H");

        let line: String = iter::repeat_n(Self::BORDER, self.full_size.w).collect();
        let side = format!("\n\r#\x1B[{}C#", self.full_size.w - 2 * Self::BORDER_WIDTH);

        print!("\n\r{}", line);
        print!("{}", side.repeat(self.constent_size.h));
        print!("\n\r{}", line);
        io::stdout().flush()
    }

    fn exit(&self) -> Result<(), Self::Error> {
        print!("\x1B[?1049l");
        print!("\x1b[?25h");
        io::stdout().flush()
    }

    fn draw<I>(&mut self, items: I) -> Result<(), Self::Error>
    where
        I: Iterator<Item = char>,
    {
        println!("\x1B[H");
        for (i, v) in items.enumerate() {
            if i % self.constent_size.w == 0 {
                print!("\n\r\x1B[{}C", Self::BORDER_WIDTH);
            }
            print!("{} ", v);
        }
        io::stdout().flush()
    }
}

impl Drop for TermScreen {
    fn drop(&mut self) {
        self.exit().unwrap();
    }
}

impl Renderble for Status {
    type Primitive = char;
    fn render(&self) -> impl Iterator<Item = Self::Primitive> {
        let owned = format!("Status: {:>4} Difficulty: {:>4}", self.score, self.diff)
            .chars()
            .collect::<Vec<_>>();
        owned.into_iter()
    }
}

pub struct TermStatusLine {
    w: usize,
}

impl TermStatusLine {
    pub fn new(w: usize) -> Self {
        Self { w: (w + 1) * 2 }
    }
}

impl RenderTarget<char> for TermStatusLine {
    type Error = io::Error;

    fn init(&self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn exit(&self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn draw<I>(&mut self, items: I) -> Result<(), Self::Error>
    where
        I: Iterator<Item = char>,
    {
        print!("\x1B[H");
        print!("{}", items.take(self.w).collect::<String>());
        io::stdout().flush()
    }
}
