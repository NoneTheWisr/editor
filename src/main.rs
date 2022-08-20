use crossterm::{event, terminal};
use editor::*;
use std::env::args;

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

    let (cols, rows) = terminal::size().unwrap();
    let term_rect = Rectangle {
        row: 0,
        col: 0,
        width: cols as _,
        height: rows as _,
    };

    let lines = buffer.screen(&term_rect).lines;
    let (last, rest) = lines.split_last().unwrap();

    rest.iter().for_each(|line| print!("{line}\r\n"));
    print!("{last}");

    event::read().unwrap();
}
