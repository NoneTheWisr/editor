use std::io::Write;

pub struct Surface {
    rect: Rectangle,
    cells: Vec<Cell>,
}

impl Surface {
    pub fn empty<R: Into<Rectangle>>(rect: R) -> Self {
        let rect = rect.into();
        let cells = vec![Cell::empty(); rect.width * rect.height];

        Self { rect, cells }
    }

    pub fn put_string(
        &mut self,
        string: impl AsRef<str>,
        style: Style,
        position: impl Into<Position>,
    ) {
        let position = position.into();
        assert!(
            self.rect.contains(position),
            "provided position is outside the surface rectangle"
        );

        let string = string.as_ref();

        let Position(x, y) = position;
        let start_offset = y * self.rect.width + x;
        let next_line_offset = (y + 1) * self.rect.width;
        let end_offset = std::cmp::min(next_line_offset, start_offset + string.len());

        self.cells.splice(
            start_offset..end_offset,
            string
                .chars()
                .take(end_offset - start_offset)
                .map(|glyph| Cell { glyph, style }),
        );
    }

    pub fn render(&self, stdout: &mut impl Write) -> std::io::Result<()> {
        use crossterm::{cursor::MoveTo, queue, style::Print};

        assert!(
            matches!(self.rect, Rectangle { x: 0, y: 0, .. }),
            "attempted to render a non-fullscreen rectangle"
        );

        let Rectangle {
            x,
            y,
            width,
            height,
        } = self.rect;

        let (left, top) = (x as u16, y as u16);

        // TODO: render the style
        let mut i = 0;
        for y in 0..height {
            queue!(stdout, MoveTo(left, top + y as u16))?;
            for _x in 0..width {
                let cell = self.cells[i];
                queue!(stdout, Print(cell.glyph))?;
                i += 1;
            }
        }

        stdout.flush()?;

        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct Position(usize, usize);

impl From<(usize, usize)> for Position {
    fn from((x, y): (usize, usize)) -> Self {
        Self(x, y)
    }
}

#[derive(Clone, Copy)]
struct Cell {
    glyph: char,
    style: Style,
}

impl Cell {
    pub fn empty() -> Self {
        Self {
            glyph: ' ',
            style: Style::default(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Style {
    bg: Color,
    fg: Color,
    formatting: Formatting,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            bg: Color::BLACK,
            fg: Color::WHITE,
            formatting: Formatting::None,
        }
    }
}

#[derive(Clone, Copy)]
enum Formatting {
    None,
    Bold,
    Italic,
    Underline,
}

#[derive(Clone, Copy)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    const BLACK: Self = Self::rgb(0, 0, 0);
    const WHITE: Self = Self::rgb(255, 255, 255);

    const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

pub struct Rectangle {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

impl Rectangle {
    pub fn left(&self) -> usize {
        self.x
    }

    pub fn right(&self) -> usize {
        self.x + self.width.saturating_sub(1)
    }

    pub fn top(&self) -> usize {
        self.y
    }

    pub fn bottom(&self) -> usize {
        self.y + self.height.saturating_sub(1)
    }

    pub fn contains(&self, Position(x, y): Position) -> bool {
        self.left() <= x && x <= self.right() && self.top() <= y && y <= self.bottom()
    }
}

impl From<(u16, u16)> for Rectangle {
    fn from((width, height): (u16, u16)) -> Self {
        Self {
            x: 0,
            y: 0,
            width: width as _,
            height: height as _,
        }
    }
}
