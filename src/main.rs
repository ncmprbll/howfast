use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Padding, Paragraph};
use ratatui::{DefaultTerminal, Frame};

const GLOBAL_PADDING: u16 = 1;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    ratatui::run(app)?;
    Ok(())
}

fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    loop {
        terminal.draw(render)?;
        if crossterm::event::read()?.is_key_press() {
            break Ok(());
        }
    }
}

fn render(frame: &mut Frame) {
    let block = Block::default()
        .borders(Borders::NONE)
        .padding(Padding::new(
            GLOBAL_PADDING,
            GLOBAL_PADDING,
            GLOBAL_PADDING,
            GLOBAL_PADDING,
        ));

    let text = Paragraph::new("Hello, world!")
        .style(Style::default().fg(Color::Red))
        .block(block);

    frame.render_widget(text, frame.area());
}
