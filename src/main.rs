use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Padding, Paragraph};
use ratatui::{DefaultTerminal, Frame};

const GLOBAL_PADDING: u16 = 1;

#[derive(Debug, Default)]
struct Coordinator {
    source: String,
    target: String,

    exit: bool,
}

impl Coordinator {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            if crossterm::event::read()?.is_key_press() {
                self.exit = true;
            }
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
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
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    ratatui::run(|terminal| Coordinator::default().run(terminal))?;
    Ok(())
}
