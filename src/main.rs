use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use rand::seq::IteratorRandom;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Padding, Paragraph, Wrap};
use ratatui::{DefaultTerminal, Frame};

const PADDING: u16 = 1;

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
            .padding(Padding::uniform(PADDING));

        let source = Line::from(Span::styled(
            self.source.as_str(),
            Style::default().fg(Color::White),
        ));

        let s1: Vec<&str> = self.source.split(' ').collect();
        let s2: Vec<&str> = self.target.split(' ').collect();
        let mut s3: Vec<Span> = Vec::new();
        let mut has_cursor: bool = false;

        for i in 0..s2.len() {
            if !has_cursor && i == s1.len() {
                s3.push(Span::styled(
                    match s2[i].chars().nth(0) {
                        Some(c) => c.to_string(),
                        None => continue,
                    },
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

            let mut span_group = Vec::new();
            let mut has_mistake: bool = false;

            for j in 0..target_word.len() {
                if j >= source_word.len() {
                    if !has_cursor && i == s1.len() - 1 {
                        span_group.push(Span::styled(
                            target_word[j].to_string(),
                            Style::default().fg(Color::Black).bg(Color::White),
                        ));

                        has_cursor = true;
                    } else {
                        span_group.push(Span::styled(
                            target_word[j].to_string(),
                            Style::default().fg(Color::Gray),
                        ));

                        if i != s1.len() - 1 {
                            has_mistake = true;
                        }
                    }

                    continue;
                }

                if source_word[j] != target_word[j] {
                    span_group.push(Span::styled(
                        target_word[j].to_string(),
                        Style::default().fg(Color::Red),
                    ));

                    has_mistake = true;
                } else {
                    span_group.push(Span::styled(
                        target_word[j].to_string(),
                        Style::default().fg(Color::White),
                    ))
                }
            }

            if source_word.len() > target_word.len() {
                for j in target_word.len()..source_word.len() {
                    span_group.push(Span::styled(
                        source_word[j].to_string(),
                        Style::default().fg(Color::LightRed),
                    ));

                    has_mistake = true;
                }
            }

            span_group.into_iter().for_each(|x| {
                if has_mistake {
                    s3.push(x.clone().style(x.style.underlined()));
                } else {
                    s3.push(x);
                };
            });

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

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(frame.area());

        let paragraph = Paragraph::new(target)
            .block(block)
            .wrap(Wrap { trim: true });

        let debug = Paragraph::new(vec![
            source,
            Line::from(format!("{}", paragraph.line_count(layout[0].width))),
        ]);

        frame.render_widget(paragraph, layout[0]);
        frame.render_widget(debug, layout[1]);
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
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.exit = true
            }
            KeyCode::Char('w') | KeyCode::Char('h')
                if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                if let Some(' ') = self.source.chars().last() {
                    self.source.pop();
                }

                let boundary_index = self.source.rfind(|c: char| c.is_whitespace()).unwrap_or(0);

                // Truncate to the nearest space character, while
                // also keeping it intact
                self.source.truncate(if boundary_index == 0 {
                    boundary_index
                } else {
                    boundary_index + 1
                });
            }
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
    const WORDS: usize = 256;

    let dictionary = vec![
        "the", "quick", "brown", "fox", "jumps", "over", "the", "lazy", "dog",
    ];

    let target = (0..WORDS)
        .map(|_| dictionary.iter().choose(&mut rand::rng()).unwrap())
        .collect::<Vec<&&str>>()
        .into_iter()
        .map(|&x| x)
        .collect::<Vec<&str>>()
        .join(" ");

    let mut coordinator = Coordinator {
        source: String::with_capacity(target.capacity()),
        target: target,
        exit: false,
    };

    color_eyre::install()?;
    ratatui::run(|terminal| coordinator.run(terminal))?;
    Ok(())
}
