use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Padding, Paragraph, Wrap};
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

        let s1: Vec<&str> = self.source.split(' ').collect();
        let s2: Vec<&str> = self.target.split(' ').collect();
        let mut s3: Vec<Span> = Vec::new();
        let mut has_cursor: bool = false;

        for i in 0..s2.len() {
            if !has_cursor && i == s1.len() {
                s3.push(Span::styled(
                    s2[i].chars().nth(0).unwrap().to_string(),
                    Style::default().fg(Color::Black).bg(Color::White),
                ));
                s3.push(Span::styled(&s2[i][1..], Style::default().fg(Color::Gray)));
                s3.push(Span::styled(" ".to_string(), Style::default()));

                has_cursor = true;

                continue;
            } else if i >= s1.len() {
                s3.push(Span::styled(s2[i], Style::default().fg(Color::Gray)));
                s3.push(Span::styled(" ".to_string(), Style::default()));

                continue;
            }

            let source_word: Vec<char> = s1[i].chars().collect();
            let target_word: Vec<char> = s2[i].chars().collect();

            for j in 0..target_word.len() {
                if j >= source_word.len() {
                    if !has_cursor {
                        s3.push(Span::styled(
                            target_word[j].to_string(),
                            Style::default().fg(Color::Black).bg(Color::White),
                        ));

                        has_cursor = true;
                    } else {
                        s3.push(Span::styled(
                            target_word[j].to_string(),
                            Style::default().fg(Color::Gray),
                        ));
                    }

                    continue;
                }

                if source_word[j] != target_word[j] {
                    s3.push(Span::styled(
                        target_word[j].to_string(),
                        Style::default().fg(Color::Red),
                    ))
                } else {
                    s3.push(Span::styled(
                        target_word[j].to_string(),
                        Style::default().fg(Color::White),
                    ))
                }
            }

            if source_word.len() > target_word.len() {
                for j in target_word.len()..source_word.len() {
                    s3.push(Span::styled(
                        source_word[j].to_string(),
                        Style::default().fg(Color::LightRed),
                    ))
                }
            }

            if !has_cursor && i == s1.len() - 1 {
                s3.push(Span::styled(
                    " ".to_string(),
                    Style::default().bg(Color::White),
                ));

                has_cursor = true;
            } else {
                s3.push(Span::styled(" ".to_string(), Style::default()))
            }
        }

        let target = Line::from(s3);

        let text = Paragraph::new(vec![source, target])
            .block(block)
            .wrap(Wrap { trim: true });

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
            KeyCode::Char(' ') if self.source.len() == 0 => (),
            KeyCode::Char(' ') if self.source.chars().last().is_some_and(|x| x == ' ') => (),
            KeyCode::Char(c) => self.source.push(c),
            KeyCode::Backspace => {
                self.source.pop();
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
