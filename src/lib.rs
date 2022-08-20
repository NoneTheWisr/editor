use anyhow;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    fs::File,
    io::{stdout, BufRead, BufReader},
    iter::repeat,
    path::Path,
};

pub struct AlternateScreen;

impl AlternateScreen {
    pub fn new() -> Self {
        execute!(stdout(), EnterAlternateScreen).unwrap();
        enable_raw_mode().unwrap();
        Self
    }
}

impl Drop for AlternateScreen {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
        execute!(stdout(), LeaveAlternateScreen).unwrap();
    }
}

pub struct Buffer {
    lines: Vec<String>,
}

impl Buffer {
    pub fn new() -> Self {
        Self { lines: Vec::new() }
    }

    pub fn from_path(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let file = File::open(path)?;
        let lines = BufReader::new(file).lines().collect::<Result<_, _>>()?;
        Ok(Self { lines })
    }

    pub fn screen(&self, rect: &Rectangle) -> Screen {
        // TODO: horizontal cropping
        let mut lines: Vec<String> = self
            .lines
            .iter()
            .skip(rect.row)
            .take(rect.height)
            .cloned()
            .collect();

        if !rect.fits_vertically(self.lines.len()) {
            let extra_count = rect.row + rect.height - self.lines.len();
            lines.extend(repeat(String::new()).take(extra_count));
        }

        Screen { lines }
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Offset {
    pub row: usize,
    pub col: usize,
}

pub struct Bounds {
    pub width: usize,
    pub height: usize,
}

pub struct Rectangle {
    pub row: usize,
    pub col: usize,
    pub width: usize,
    pub height: usize,
}

impl Rectangle {
    pub fn fits_vertically(&self, line_count: usize) -> bool {
        self.row + self.height <= line_count
    }
}

pub struct Screen {
    pub lines: Vec<String>,
}
