use crossterm::event::Event;
use crossterm::terminal::{Clear, ClearType};
use crossterm::{cursor::MoveTo, event, queue, style::Print, terminal};
use editor::{buffer::Buffer, terminal::AlternateScreen, Cursor, Rectangle, Screen};
use std::env::args;
use std::io::{stdout, BufWriter, Write};

fn main() {
    let _screen = AlternateScreen::new();
    let args: Vec<String> = args().skip(1).collect();

    let buffer = match args.len() {
        0 => Ok(Buffer::new()),
        1 => Buffer::from_path(&args[0]).map_err(|_| "couldn't open the file"),
        _ => Err("usage: editor [file-path]"),
    };

    let mut buffer = match buffer {
        Ok(buffer) => buffer,
        Err(description) => {
            drop(_screen);
            println!("{description}");
            return;
        }
    };

    render_screen(&buffer.screen(&terminal_rectangle())).unwrap();

    while let Ok(event) = event::read() {
        if let Event::Key(event) = event {
            use editor::buffer::CursorMovement;
            use event::KeyCode::*;

            match (event.code, event.modifiers) {
                (Esc | Char('q'), _) => break,

                (Up, _) => buffer.move_cursor(CursorMovement::Up),
                (Down, _) => buffer.move_cursor(CursorMovement::Down),
                (Left, _) => buffer.move_cursor(CursorMovement::Left),
                (Right, _) => buffer.move_cursor(CursorMovement::Right),

                _ => (),
            }

            render_screen(&buffer.screen(&terminal_rectangle())).unwrap();
        }
    }
}

fn render_screen(screen: &Screen) -> anyhow::Result<()> {
    let mut stdout = BufWriter::new(stdout());
    queue!(stdout, Clear(ClearType::All))?;
    queue!(stdout, MoveTo(0, 0))?;

    let (last_line, lines) = screen.lines.split_last().unwrap();
    for line in lines {
        queue!(stdout, Print(line))?;
        queue!(stdout, Print("\r\n"))?;
        // queue!(stdout, Clear(ClearType::UntilNewLine))?;
    }
    queue!(stdout, Print(last_line))?;
    // queue!(stdout, Clear(ClearType::FromCursorDown))?;

    let Cursor { row, col } = screen.cursor;
    queue!(stdout, MoveTo(col as u16, row as u16))?;

    stdout.flush()?;

    Ok(())
}

fn terminal_rectangle() -> Rectangle {
    let (cols, rows) = terminal::size().unwrap();
    Rectangle {
        row: 0,
        col: 0,
        width: cols as _,
        height: rows as _,
    }
}
