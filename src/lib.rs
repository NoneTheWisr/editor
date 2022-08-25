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
    use std::{
        fs::File,
        io::{BufRead, BufReader},
        path::Path,
    };

    pub struct Buffer {
        lines: Vec<String>,
        cursor: Cursor,
        view: View,
    }

    impl Buffer {
        pub fn new(view: View) -> Self {
            Self {
                lines: vec![String::new()],
                cursor: Cursor::default(),
                view,
            }
        }

        pub fn from_path(path: impl AsRef<Path>, view: View) -> anyhow::Result<Self> {
            let file = File::open(path)?;
            let reader = BufReader::new(file);

            Ok(Self {
                lines: reader.lines().collect::<Result<_, _>>()?,
                cursor: Cursor::default(),
                view,
            })
        }

        pub fn display(&self) -> crate::display::Screen {
            let lines = self
                .lines
                .iter()
                .skip(self.view.min_y())
                .take(self.view.height)
                .map(|line| {
                    line.chars()
                        .skip(self.view.min_x())
                        .take(self.view.width)
                        .collect()
                })
                .collect();
            let cursor = crate::display::Cursor {
                x: self.cursor.x - self.view.min_x(),
                y: self.cursor.y - self.view.min_y(),
            };

            crate::display::Screen { lines, cursor }
        }

        pub fn move_cursor(&mut self, movement: CursorMovement) {
            match movement {
                CursorMovement::Up => {
                    self.cursor.y = self.cursor.y.saturating_sub(1);
                    self.clamp_cursor_to_line();
                }
                CursorMovement::Down => {
                    self.cursor.y = std::cmp::min(
                        self.cursor.y.saturating_add(1),
                        self.line_count().saturating_sub(1),
                    );
                    self.clamp_cursor_to_line();
                }
                CursorMovement::Left => {
                    if self.cursor_at_line_start() {
                        if !self.cursor_at_first_line() {
                            self.cursor.y -= 1;
                            self.cursor.x = self.line_len(self.cursor.y).saturating_sub(1);
                        }
                    } else {
                        self.cursor.x -= 1;
                    }
                }
                CursorMovement::Right => {
                    if self.cursor_at_line_end() {
                        if !self.cursor_at_last_line() {
                            self.cursor.y += 1;
                            self.cursor.x = 0;
                        }
                    } else {
                        self.cursor.x += 1;
                    }
                }
            }

            self.adjust_view()
        }

        fn adjust_view(&mut self) {
            if self.cursor.y < self.view.min_y() {
                self.view.y = self.cursor.y;
            } else if self.cursor.y > self.view.max_y() {
                self.view.y += self.cursor.y - self.view.max_y();
            }

            if self.cursor.x < self.view.min_x() {
                self.view.x = self.cursor.x;
            } else if self.cursor.x > self.view.max_x() {
                self.view.x += self.cursor.x - self.view.max_x();
            }
        }

        fn line_count(&self) -> usize {
            self.lines.len()
        }

        fn line_len(&self, line_number: usize) -> usize {
            self.lines[line_number].len()
        }

        fn cursor_at_line_start(&self) -> bool {
            self.cursor.x == 0
        }

        fn cursor_at_line_end(&self) -> bool {
            self.cursor.x == self.lines[self.cursor.y].len().saturating_sub(1)
        }

        fn cursor_at_first_line(&self) -> bool {
            self.cursor.y == 0
        }

        fn cursor_at_last_line(&self) -> bool {
            self.cursor.y == self.lines.len().saturating_sub(1)
        }

        fn clamp_cursor_to_line(&mut self) {
            let max_col = self.lines[self.cursor.y].len();
            self.cursor.x = std::cmp::min(self.cursor.x, max_col);
        }
    }

    #[derive(Default)]
    pub struct Cursor {
        x: usize,
        y: usize,
    }

    impl Cursor {
        pub fn new(x: usize, y: usize) -> Self {
            Self { x, y }
        }
    }

    pub struct View {
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    }

    impl View {
        pub fn new(width: usize, height: usize) -> Self {
            Self {
                x: 0,
                y: 0,
                width,
                height,
            }
        }

        pub fn min_x(&self) -> usize {
            self.x
        }

        pub fn max_x(&self) -> usize {
            self.x + self.width.saturating_sub(1)
        }

        pub fn min_y(&self) -> usize {
            self.y
        }

        pub fn max_y(&self) -> usize {
            self.y + self.height.saturating_sub(1)
        }
    }

    pub enum CursorMovement {
        Up,
        Down,
        Left,
        Right,
    }
}

pub mod display {
    pub struct Screen {
        pub lines: Vec<String>,
        pub cursor: Cursor,
    }

    pub struct Cursor {
        pub x: usize,
        pub y: usize,
    }

    impl Cursor {
        pub fn new(x: usize, y: usize) -> Self {
            Self { x, y }
        }
    }
}
