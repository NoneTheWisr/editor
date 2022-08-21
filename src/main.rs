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

    let buffer = match buffer {
        Ok(buffer) => buffer,
        Err(description) => {
            drop(_screen);
            println!("{description}");
            return;
        }
    };

    render_screen(&buffer.screen(&terminal_rectangle())).unwrap();

    event::read().unwrap();
}

fn render_screen(screen: &Screen) -> anyhow::Result<()> {
    let mut stdout = BufWriter::new(stdout());

    let (last_line, lines) = screen.lines.split_last().unwrap();
    for line in lines {
        queue!(stdout, Print(line))?;
        queue!(stdout, Print("\r\n"))?;
    }
    queue!(stdout, Print(last_line))?;

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
