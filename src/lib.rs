pub mod terminal_ui;

mod display {
    pub struct Screen {
        dimensions: Dimensions,
        lines: Vec<String>,
        cursor: Cursor,
    }

    struct Dimensions {
        width: usize,
        height: usize,
    }

    struct Cursor {
        line: usize,
        column: usize,
    }
}

mod terminal_display {
    trait Displayable {
        fn display(&self) -> crate::terminal_ui::Surface;
    }

    impl Displayable for crate::display::Screen {
        fn display(&self) -> crate::terminal_ui::Surface {
            todo!()
        }
    }
}
