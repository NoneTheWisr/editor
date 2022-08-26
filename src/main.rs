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

    let mut mode = Mode::Normal;

    while let Ok(event) = event::read() {
        if let Event::Key(event) = event {
            use editor::buffer::CursorMovement;
            use event::{KeyCode::*, KeyModifiers};

            match (mode, event.code, event.modifiers) {
                (Mode::Normal, Char('q'), _) => break,

                (Mode::Insert, Esc, _) => mode = Mode::Normal,
                (Mode::Normal, Char('i'), _) => mode = Mode::Insert,

                (Mode::Normal, Char('d'), _) => buffer.remove_char(),

                (Mode::Normal, Char('k'), _) => buffer.move_cursor(CursorMovement::Up),
                (Mode::Normal, Char('j'), _) => buffer.move_cursor(CursorMovement::Down),
                (Mode::Normal, Char('h'), _) => buffer.move_cursor(CursorMovement::Left),
                (Mode::Normal, Char('l'), _) => buffer.move_cursor(CursorMovement::Right),

                (_, Home, _) => buffer.move_cursor(CursorMovement::LineStart),
                (_, End, _) => buffer.move_cursor(CursorMovement::LineEnd),

                (Mode::Insert, Delete, _) => buffer.remove_char(),
                (Mode::Insert, Backspace, _) => {
                    buffer.move_cursor(CursorMovement::Left);
                    buffer.remove_char();
                }

                (Mode::Insert, Enter, _) => buffer.insert_line(),

                (Mode::Insert, Char(c), KeyModifiers::SHIFT) => {
                    buffer.insert_char(c.to_ascii_uppercase())
                }
                (Mode::Insert, Char(c), _) => buffer.insert_char(c),

                _ => (),
            }

            render_screen(&buffer.display())?;
        }
    }

    Ok(())
}

#[derive(Clone, Copy)]
enum Mode {
    Normal,
    Insert,
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
