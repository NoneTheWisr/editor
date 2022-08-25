use crossterm::event::Event;
use crossterm::terminal::{Clear, ClearType};
use crossterm::{cursor::MoveTo, event, queue, style::Print, terminal};
use editor::{
    buffer::{Buffer, View},
    display::{Cursor, Screen},
    terminal::AlternateScreen,
};
use std::env::args;
use std::io::{stdout, BufWriter, Write};

fn main() -> anyhow::Result<()> {
    let _screen = AlternateScreen::new();
    let args: Vec<String> = args().skip(1).collect();

    let (width, height) = terminal::size()?;
    let view = View::new(width as _, height as _);

    let buffer = match args.len() {
        0 => Ok(Buffer::new(view)),
        1 => Buffer::from_path(&args[0], view).map_err(|_| "couldn't open the file"),
        _ => Err("usage: editor [file-path]"),
    };

    let mut buffer = match buffer {
        Ok(buffer) => buffer,
        Err(description) => {
            drop(_screen);
            println!("{description}");
            return Ok(());
        }
    };

    render_screen(&buffer.display())?;

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

            render_screen(&buffer.display())?;
        }
    }

    Ok(())
}

fn render_screen(screen: &Screen) -> anyhow::Result<()> {
    use anyhow::Context;

    let mut stdout = BufWriter::new(stdout());
    queue!(stdout, Clear(ClearType::All))?;
    queue!(stdout, MoveTo(0, 0))?;

    let (last_line, lines) = screen.lines.split_last().context("coulnd't split")?;
    for line in lines {
        queue!(stdout, Print(line))?;
        queue!(stdout, Print("\r\n"))?;
    }
    queue!(stdout, Print(last_line))?;

    let Cursor { x, y } = screen.cursor;
    queue!(stdout, MoveTo(x as u16, y as u16))?;

    stdout.flush()?;

    Ok(())
}
