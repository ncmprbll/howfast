use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
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
            self.handle_events()?;
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

        let source = Line::from(Span::styled(
            self.source.as_str(),
            Style::default().fg(Color::White),
        ));
        let target = Line::from(Span::styled(
            self.target.as_str(),
            Style::default().fg(Color::Gray),
        ));

        let text = Paragraph::new(vec![source, target]).block(block);

        frame.render_widget(text, frame.area());
    }

    fn handle_events(&mut self) -> std::io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => self.exit = true,
            KeyCode::Char(c) => {
                if c == ' ' && (self.source.len() == 0 || self.source.chars().last().is_some()) {
                    return;
                }

                self.source.push(c);
            }
            _ => {}
        }
    }
}

fn main() -> color_eyre::Result<()> {
    let mut coordinator = Coordinator {
        source: String::new(),
        target: String::from("The quick brown fox jumps over the lazy dog."),
        exit: false,
    };

    color_eyre::install()?;
    ratatui::run(|terminal| coordinator.run(terminal))?;
    Ok(())
}
