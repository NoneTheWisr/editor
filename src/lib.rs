pub mod terminal {
    use crossterm::{
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use std::io::stdout;

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
}

pub mod buffer {
    use crate::{Rectangle, Screen};
    use anyhow;
    use std::{
        fs::File,
        io::{BufRead, BufReader},
        iter::repeat,
        path::Path,
    };

    pub struct Buffer {
        lines: Vec<String>,
        cursor: Cursor,
    }

    impl Buffer {
        pub fn new() -> Self {
            Self {
                lines: Vec::new(),
                cursor: Cursor::default(),
            }
        }

        pub fn from_path(path: impl AsRef<Path>) -> anyhow::Result<Self> {
            let file = File::open(path)?;
            let lines = BufReader::new(file).lines().collect::<Result<_, _>>()?;
            let cursor = Cursor::default();
            Ok(Self { lines, cursor })
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

            let cursor = crate::Cursor::default();

            Screen { lines, cursor }
        }
    }

    impl Default for Buffer {
        fn default() -> Self {
            Self::new()
        }
    }

    struct Cursor {
        pub row: usize,
        pub col: usize,
    }

    impl Cursor {
        pub fn new(row: usize, col: usize) -> Self {
            Self { row, col }
        }
    }

    impl Default for Cursor {
        fn default() -> Self {
            Self::new(0, 0)
        }
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

pub struct Cursor {
    pub row: usize,
    pub col: usize,
}

impl Cursor {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

pub struct Screen {
    pub lines: Vec<String>,
    pub cursor: Cursor,
}
