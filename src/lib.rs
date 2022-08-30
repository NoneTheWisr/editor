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
        cmp::min,
        fs,
        path::{Path, PathBuf},
    };

    pub struct Buffer {
        lines: Vec<String>,
        cursor: Cursor,
        view: View,
        file_path: Option<PathBuf>,
    }

    impl Buffer {
        pub fn new(view: View) -> Self {
            Self {
                lines: vec![String::new()],
                cursor: Cursor::default(),
                view,
                file_path: None,
            }
        }

        pub fn from_path(path: impl AsRef<Path>, view: View) -> anyhow::Result<Self> {
            let path = path.as_ref().to_path_buf();

            Ok(Self {
                lines: (fs::read_to_string(&path)? + "\n").lines().map(str::to_string).collect(),
                cursor: Cursor::default(),
                view,
                file_path: Some(path),
            })
        }

        pub fn save(&mut self) -> std::io::Result<()> {
            self.save_as(&self.file_path.clone().unwrap())
        }

        pub fn save_as(&mut self, path: impl AsRef<Path>) -> std::io::Result<()> {
            let path = path.as_ref().to_path_buf();
            std::fs::write(&path, self.lines.join("\n"))?;
            self.file_path = Some(path);
            Ok(())
        }

        pub fn view(&self) -> &View {
            &self.view
        }

        pub fn cursor(&self) -> &Cursor {
            &self.cursor
        }

        pub fn line_count(&self) -> usize {
            self.lines.len()
        }

        pub fn file_path(&self) -> Option<&Path> {
            self.file_path.as_deref()
        }

        pub fn display(&self) -> crate::display::Screen {
            use std::iter::repeat;
            let empty_count = (self.view.max_y() + 1)
                .saturating_sub(self.line_count());
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
                .chain(repeat(String::from("")).take(empty_count))
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
                    if !self.cursor_at_first_line() {
                        self.cursor.move_up();
                    }
                    self.clamp_cursor_to_line_boundaries();
                }
                CursorMovement::Down => {
                    if !self.cursor_at_last_line() {
                        self.cursor.move_down();
                    }
                    self.clamp_cursor_to_line_boundaries();
                }
                CursorMovement::Left => {
                    if self.cursor_at_line_start() {
                        if !self.cursor_at_first_line() {
                            self.cursor.move_up();
                            self.cursor.x = self.line_len(self.cursor.y);
                        }
                    } else {
                        self.cursor.move_left();
                    }
                }
                CursorMovement::Right => {
                    if self.cursor_at_line_end() {
                        if !self.cursor_at_last_line() {
                            self.cursor.move_to_start_of_next_line();
                        }
                    } else {
                        self.cursor.move_right();
                    }
                }
                CursorMovement::LineStart => {
                    self.cursor.move_to_start_of_line();
                }
                CursorMovement::LineEnd => {
                    self.cursor.x = self.line_len(self.cursor.y);
                }
                CursorMovement::TextStart => {
                    self.cursor.x = self.lines[self.cursor.y]
                        .find(|c: char| !c.is_whitespace()).unwrap_or_default();
                }
                CursorMovement::FirstLine => {
                    self.cursor.y = 0;
                    self.cursor.move_to_start_of_line();
                }
                CursorMovement::LastLine => {
                    self.cursor.y = self.line_count().saturating_sub(1);
                    self.cursor.move_to_start_of_line();
                }
                CursorMovement::ToLine(line_number) => {
                    self.cursor.y = line_number;
                    self.cursor.move_to_start_of_line();
                }
            }

            self.adjust_view();
        }

        pub fn insert_char(&mut self, character: char) {
            self.lines[self.cursor.y].insert(self.cursor.x, character);
            self.cursor.move_right();
            self.adjust_view();
        }

        pub fn insert_string(&mut self, string: &str) {
            self.lines[self.cursor.y].insert_str(self.cursor.x, string);
            self.cursor.x += string.len();
            self.adjust_view();
        }

        pub fn remove_char(&mut self) {
            if self.cursor_at_line_end() {
                if !self.cursor_at_last_line() {
                    self.join_lines(self.cursor.y, self.cursor.y + 1);
                }
            } else {
                self.lines[self.cursor.y].remove(self.cursor.x);
            }
        }

        pub fn insert_line(&mut self) {
            self.lines.insert(
                self.cursor.y + 1,
                self.lines[self.cursor.y][self.cursor.x..].to_string(),
            );
            self.lines[self.cursor.y].replace_range(self.cursor.x.., "");

            self.cursor.move_to_start_of_next_line();
            self.adjust_view();
        }

        pub fn remove_line(&mut self) {
            let should_move_cursor_up =
                self.cursor_at_last_line() && self.line_count() > 1;

            self.lines.remove(self.cursor.y);
            self.cursor.move_to_start_of_line();

            if should_move_cursor_up {
                self.cursor.move_up();
            }
            if self.lines.is_empty() {
                self.lines.push(String::new());
            }
        }

        pub fn join_lines(&mut self, first: usize, last: usize) {
            self.lines.splice(
                first..=last,
                std::iter::once(
                    self.lines[first..=last]
                        .into_iter()
                        .flat_map(|line| line.chars())
                        .collect::<String>(),
                ),
            );
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

        fn line_len(&self, line_number: usize) -> usize {
            self.lines[line_number].len()
        }

        fn cursor_at_line_start(&self) -> bool {
            self.cursor.x == 0
        }

        fn cursor_at_line_end(&self) -> bool {
            self.cursor.x == self.lines[self.cursor.y].len()
        }

        fn cursor_at_first_line(&self) -> bool {
            self.cursor.y == 0
        }

        fn cursor_at_last_line(&self) -> bool {
            self.cursor.y == self.lines.len().saturating_sub(1)
        }

        fn clamp_cursor_to_line_boundaries(&mut self) {
            let max_col = self.lines[self.cursor.y].len();
            self.cursor.x = min(self.cursor.x, max_col);
        }
    }

    #[derive(Default)]
    pub struct Cursor {
        pub x: usize,
        pub y: usize,
    }

    // I like the idea of these methods, but I don't like how
    // verbose the naming is. I also dislike that not all of
    // such methods can be on the cursor. Some have to be on
    // the buffer, because line lengths and line count must
    // be known (those can be passed as parameters, but that
    // seems a little silly.
    impl Cursor {
        pub fn move_right(&mut self) {
            self.x += 1;
        }

        pub fn move_left(&mut self) {
            self.x -= 1;
        }

        pub fn move_up(&mut self) {
            self.y -= 1;
        }

        pub fn move_down(&mut self) {
            self.y += 1;
        }

        pub fn move_to_start_of_line(&mut self) {
            self.x = 0;
        }

        pub fn move_to_start_of_next_line(&mut self) {
            self.move_down();
            self.move_to_start_of_line();
        }
    }

    impl Cursor {
        pub fn new(x: usize, y: usize) -> Self {
            Self { x, y }
        }
    }

    pub struct View {
        pub x: usize,
        pub y: usize,
        pub width: usize,
        pub height: usize,
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
        LineStart,
        LineEnd,
        TextStart,
        FirstLine,
        LastLine,
        ToLine(usize),
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