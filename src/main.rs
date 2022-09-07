fn main() -> std::io::Result<()> {
    let dimensions = crossterm::terminal::size()?;
    let (width, height) = dimensions;
    let rectangle = editor::terminal_ui::Rectangle::from(dimensions);
    let mut surface = editor::terminal_ui::Surface::empty(rectangle);

    crossterm::terminal::enable_raw_mode()?;

    surface.draw_string(
        "test",
        editor::terminal_ui::Style::default(),
        ((width - 4) as _, (height - 1) as _),
    );
    surface.render(&mut std::io::stdout())?;

    std::thread::sleep_ms(1000);

    crossterm::event::read()?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}
