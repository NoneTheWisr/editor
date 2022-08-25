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
    use crate::display::Rectangle;
    use anyhow;
    use std::{
        fs::File,
        io::{BufRead, BufReader},
        iter::repeat,
        path::Path,
    };

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

        pub fn rect(&self, rect: &Rectangle) -> Vec<String> {
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

            lines
        }

        pub fn line_count(&self) -> usize {
            self.lines.len()
        }

        pub fn line_length(&self, line_number: usize) -> usize {
            self.lines[line_number].len()
        }

        pub fn is_at_line_start(&self, cursor: &Cursor) -> bool {
            cursor.col == 0
        }

        pub fn is_at_line_end(&self, cursor: &Cursor) -> bool {
            cursor.col == self.lines[cursor.row].len().saturating_sub(1)
        }

        pub fn is_at_first_line(&self, cursor: &Cursor) -> bool {
            cursor.row == 0
        }

        pub fn is_at_last_line(&self, cursor: &Cursor) -> bool {
            cursor.row == self.lines.len().saturating_sub(1)
        }

        pub fn clamp(&self, mut cursor: Cursor) -> Cursor {
            let max_col = self.lines[cursor.row].len();
            cursor.col = std::cmp::min(cursor.col, max_col);
            cursor
        }

        pub fn move_cursor(
            &self,
            mut cursor: Cursor,
            movement: crate::logic::CursorMovement,
        ) -> Cursor {
            use crate::logic::CursorMovement;
            use std::cmp::min;

            match movement {
                CursorMovement::Up => {
                    cursor.row = cursor.row.saturating_sub(1);
                    cursor = self.clamp(cursor);
                }
                CursorMovement::Down => {
                    cursor.row = min(self.line_count(), cursor.row.saturating_add(1));
                    cursor = self.clamp(cursor);
                }
                CursorMovement::Left => {
                    if self.is_at_line_start(&cursor) {
                        if !self.is_at_first_line(&cursor) {
                            cursor.row -= 1;
                            cursor.col = self.line_length(cursor.row).saturating_sub(1);
                        }
                    } else {
                        cursor.col -= 1;
                    }
                }
                CursorMovement::Right => {
                    if self.is_at_line_end(&cursor) {
                        if !self.is_at_last_line(&cursor) {
                            cursor.row += 1;
                            cursor.col = 0;
                        }
                    } else {
                        cursor.col += 1;
                    }
                }
            }

            cursor
        }
    }

    impl Default for Buffer {
        fn default() -> Self {
            Self::new()
        }
    }

    #[derive(Clone, Copy, Default)]
    pub struct Cursor {
        pub row: usize,
        pub col: usize,
    }

    impl Cursor {
        pub fn new(row: usize, col: usize) -> Self {
            Self { row, col }
        }
    }

    impl From<crate::logic::Cursor> for Cursor {
        fn from(cursor: crate::logic::Cursor) -> Self {
            Self::new(cursor.row, cursor.col)
        }
    }
}

pub mod logic {
    pub struct View {
        pub row: usize,
        pub col: usize,
        pub width: usize,
        pub height: usize,
    }

    impl View {
        pub fn with_dimensions(width: usize, height: usize) -> Self {
            Self {
                row: 0,
                col: 0,
                width,
                height,
            }
        }

        pub fn first_row(&self) -> usize {
            self.row
        }

        pub fn last_row(&self) -> usize {
            self.row + self.height.saturating_sub(1)
        }

        pub fn first_col(&self) -> usize {
            self.col
        }

        pub fn last_col(&self) -> usize {
            self.col + self.width.saturating_sub(1)
        }
    }

    #[derive(Clone, Copy, Default)]
    pub struct Cursor {
        pub row: usize,
        pub col: usize,
    }

    impl Cursor {
        pub fn new(row: usize, col: usize) -> Self {
            Self { row, col }
        }
    }

    impl From<crate::buffer::Cursor> for Cursor {
        fn from(cursor: crate::buffer::Cursor) -> Self {
            Self::new(cursor.row, cursor.col)
        }
    }

    pub enum CursorMovement {
        Up,
        Down,
        Left,
        Right,
    }

    pub fn move_cursor_with_view(
        buffer: &crate::buffer::Buffer,
        mut cursor: Cursor,
        mut view: View,
        movement: CursorMovement,
    ) -> (Cursor, View) {
        cursor = buffer.move_cursor(cursor.into(), movement).into();

        if cursor.row < view.first_row() {
            view.row = cursor.row;
        } else if cursor.row > view.last_row() {
            view.row += cursor.row - view.last_row();
        }

        if cursor.col < view.first_col() {
            view.col = cursor.col;
        } else if cursor.col > view.last_col() {
            view.col += cursor.col - view.last_col();
        }

        (cursor, view)
    }
}

pub mod display {
    pub struct Screen {
        pub lines: Vec<String>,
        pub cursor: Cursor,
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

    impl From<crate::logic::View> for Rectangle {
        fn from(view: crate::logic::View) -> Self {
            Self {
                row: view.row,
                col: view.col,
                width: view.width,
                height: view.height,
            }
        }
    }
}
