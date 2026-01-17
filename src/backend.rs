#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub w: usize,
    pub h: usize,
}

impl Size {
    pub fn new(w: usize, h: usize) -> Size {
        Size { w, h }
    }

    pub fn middle(&self) -> usize {
        self.h / 2 * self.w + self.w / 2
    }

    pub fn flatten(&self) -> usize {
        self.h * self.w
    }
}

pub trait Renderble {
    type Primitive;
    fn render(&self) -> impl Iterator<Item = Self::Primitive>;
}

pub trait RenderTarget<P> {
    type Error;

    fn init(&self) -> Result<(), Self::Error>;
    fn exit(&self) -> Result<(), Self::Error>;
    fn draw<I>(&mut self, items: I) -> Result<(), Self::Error>
    where
        I: Iterator<Item = P>;
}

#[derive(Eq, PartialEq)]
pub enum Quad {
    Left,
    Center,
    Right,
}
