use crossterm::event::Event;
use crossterm::terminal::{Clear, ClearType};
use crossterm::{cursor::MoveTo, event, queue, style::Print, terminal};
use editor::{
    buffer::{Buffer, View},
    display::{Cursor, Screen},
    terminal::AlternateScreen,
};
use std::env;
use std::io::{stdout, BufWriter, Write};

fn main() -> anyhow::Result<()> {
    let _screen = AlternateScreen::new();
    let args: Vec<String> = env::args().skip(1).collect();

    let view = make_view()?;
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

    let mut mode = Mode::Normal;
    render(&buffer, &mode)?;

    while let Ok(event) = event::read() {
        if let Event::Key(event) = event {
            use editor::buffer::CursorMovement;
            use event::{KeyCode::*, KeyModifiers};

            match (mode, event.code, event.modifiers) {
                (Mode::Normal, Char('q'), _) => break,

                (Mode::Insert, Esc, _) => mode = Mode::Normal,
                (Mode::Normal, Char('i'), _) => mode = Mode::Insert,
                (Mode::Normal, Char('a'), _) => {
                    buffer.move_cursor(CursorMovement::Right);
                    mode = Mode::Insert
                }
                (Mode::Normal, Char('I'), _) => {
                    buffer.move_cursor(CursorMovement::TextStart);
                    mode = Mode::Insert
                }
                (Mode::Normal, Char('A'), _) => {
                    buffer.move_cursor(CursorMovement::LineEnd);
                    mode = Mode::Insert
                }
                (Mode::Normal, Char('o'), _) => {
                    buffer.move_cursor(CursorMovement::LineEnd);
                    buffer.insert_line();
                    mode = Mode::Insert
                }

                (Mode::Normal, Char('d'), _) => buffer.remove_char(),
                (Mode::Normal, Char('D'), _) => buffer.remove_line(),

                (Mode::Normal, Char('k'), _) => buffer.move_cursor(CursorMovement::Up),
                (Mode::Normal, Char('j'), _) => buffer.move_cursor(CursorMovement::Down),
                (Mode::Normal, Char('h'), _) => buffer.move_cursor(CursorMovement::Left),
                (Mode::Normal, Char('l'), _) => buffer.move_cursor(CursorMovement::Right),

                (Mode::Normal, Char('K'), _) => buffer.move_cursor(CursorMovement::FirstLine),
                (Mode::Normal, Char('J'), _) => buffer.move_cursor(CursorMovement::LastLine),
                (Mode::Normal, Char('H'), _) => buffer.move_cursor(CursorMovement::LineStart),
                (Mode::Normal, Char('L'), _) => buffer.move_cursor(CursorMovement::LineEnd),
                (Mode::Normal, Char('S'), _) => buffer.move_cursor(CursorMovement::TextStart),

                (Mode::Normal, Char(':'), _) => read_command(&mut buffer)?,

                (_, Home, _) => buffer.move_cursor(CursorMovement::LineStart),
                (_, End, _) => buffer.move_cursor(CursorMovement::LineEnd),

                (Mode::Insert, Delete, _) => buffer.remove_char(),
                (Mode::Insert, Backspace, _) => {
                    buffer.move_cursor(CursorMovement::Left);
                    buffer.remove_char();
                }

                (Mode::Insert, Enter, _) => buffer.insert_line(),

                (Mode::Insert, Tab, _) => buffer.insert_string("    "),
                (Mode::Insert, Char(c), KeyModifiers::SHIFT) => {
                    buffer.insert_char(c.to_ascii_uppercase())
                }
                (Mode::Insert, Char(c), _) => buffer.insert_char(c),

                _ => (),
            }

            render(&buffer, &mode)?;
        } else if let Event::Resize(_, _) = event {
            render(&buffer, &mode)?;
        }
    }

    Ok(())
}

#[derive(Clone, Copy)]
enum Mode {
    Normal,
    Insert,
}

fn make_view() -> anyhow::Result<View> {
    let (width, height) = terminal::size()?;
    let height = height.saturating_sub(1);
    Ok(View::new(width as _, height as _))
}

fn read_command(buffer: &mut Buffer) -> anyhow::Result<()> {
    let mut command = String::new();
    render_command_prompt(&command)?;

    while let Ok(event) = event::read() {
        if let Event::Key(event) = event {
            use event::{KeyCode::*, KeyModifiers};

            match (event.code, event.modifiers) {
                (Esc, _) => break,

                (Char(c), KeyModifiers::SHIFT) => command.push(c.to_ascii_uppercase()),
                (Char(c), _) => command.push(c),

                (Backspace, _) => {
                    command.pop();
                }

                (Enter, _) => return process_command(buffer, command),

                _ => (),
            }

            render_command_prompt(&command)?;
        } else if let Event::Resize(_, _) = event {
            render_command_prompt(&command)?;
        }
    }

    Ok(())
}

fn process_command(buffer: &mut Buffer, command: String) -> anyhow::Result<()> {
    use editor::buffer::CursorMovement;

    if command == "w" {
        buffer.save()?;
    } else if command.starts_with("w ") {
        buffer.save_as(shellexpand::full(&command[2..])?.as_ref())?;
    } else if command.starts_with("o ") {
        *buffer = Buffer::from_path(shellexpand::full(&command[2..])?.as_ref(), make_view()?)?;
    } else if command.starts_with("cd "){
        env::set_current_dir(shellexpand::full(&command[3..])?.as_ref())?;
    } else if command.starts_with("g ") {
        let line_number: usize = command[2..].parse()?;
        buffer.move_cursor(CursorMovement::ToLine(line_number.saturating_sub(1)));
    }

    Ok(())
}

fn render(buffer: &Buffer, mode: &Mode) -> anyhow::Result<()> {
    use crossterm::style::Stylize;

    let mut stdout = BufWriter::new(stdout());

    queue!(stdout, MoveTo(0, buffer.view().height as _))?;
    let status = match mode {
        Mode::Normal => "NORMAL",
        Mode::Insert => "INSERT",
    };
    let path = match buffer.file_path() {
        Some(path) => path.to_string_lossy().into_owned(),
        None => String::from("[scratch]"),
    };
    let editor::buffer::Cursor { x, y } = buffer.cursor();
    let count = buffer.line_count();
    let line_info = format!("{}:{}/{count}", x + 1, y + 1);
    let status = format!(" <{status}> [{line_info}] {path}");
    queue!(
        stdout,
        Print(format!("{status:0$.0$}", buffer.view().width as _).on_dark_grey())
    )?;

    render_screen(&buffer.display(), &mut stdout)?;

    stdout.flush()?;

    Ok(())
}

fn render_screen(screen: &Screen, stdout: &mut impl Write) -> anyhow::Result<()> {
    use anyhow::Context;

    queue!(stdout, MoveTo(0, 0))?;

    // TODO: remove the context.
    let (last_line, lines) = screen.lines.split_last().context("coulnd't split")?;
    for line in lines {
        queue!(stdout, Print(line))?;
        queue!(stdout, Clear(ClearType::UntilNewLine))?;
        queue!(stdout, Print("\r\n"))?;
    }
    queue!(stdout, Print(last_line))?;
    queue!(stdout, Clear(ClearType::UntilNewLine))?;

    let Cursor { x, y } = screen.cursor;
    queue!(stdout, MoveTo(x as u16, y as u16))?;

    Ok(())
}

fn render_command_prompt(command: &str) -> anyhow::Result<()> {
    use crossterm::cursor::MoveToColumn;
    use crossterm::style::Stylize;

    let (width, height) = terminal::size()?;

    let mut stdout = BufWriter::new(stdout());
    queue!(stdout, MoveTo(0, height - 1))?;
    queue!(stdout, Clear(ClearType::UntilNewLine))?;

    queue!(
        stdout,
        Print(format!(":{command:0$.0$}", (width - 1) as _).on_dark_grey())
    )?;
    queue!(stdout, MoveToColumn((command.len() + 1) as _))?;

    stdout.flush()?;

    Ok(())
}
