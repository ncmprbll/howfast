use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Padding, Paragraph, Wrap};
use ratatui::{DefaultTerminal, Frame};
use std::ops::{Add, Sub};
use std::time::{Duration, Instant};

mod dictionary;
use dictionary::Dictionary;

const PADDING: u16 = 1;
/// The amount of seconds you get for typing
const TYPING_TIME: Duration = Duration::from_secs(999);
const POLLING_TIME: Duration = Duration::from_millis(200);
/// Amount of lines to render at a time
/// TODO: Implement an actual limit
const RENDER_LINES: usize = 3;
const REQUEST_LINES_THRESHOLD: usize = RENDER_LINES + 1;
const TARGET_TEXT_SIZE_LIMIT: usize = 1 << 20;

#[derive(Debug)]
struct Coordinator {
    dictionary: Dictionary,

    source: String,
    target: String,
    ends_at: Instant,

    exit: bool,
    started: bool,
    need_more_lines: bool,
}

impl Coordinator {
    /// How many words to request on initial creation
    /// and future expansions
    const REQUEST_WORDS: usize = 64;

    fn new(dictionary: Dictionary) -> Self {
        let target = dictionary.get_random_words(Self::REQUEST_WORDS).join(" ");

        Self {
            dictionary,
            source: String::with_capacity(target.len()),
            target,
            ends_at: Instant::now(),
            exit: false,
            started: false,
            need_more_lines: false,
        }
    }

    fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
    ) -> std::io::Result<(f64, &str, usize, usize, usize, usize)> {
        while !self.exit {
            if self.started && Instant::now() >= self.ends_at {
                self.exit = true;
                continue;
            }

            if self.need_more_lines {
                let additional_text = self
                    .dictionary
                    .get_random_words(Self::REQUEST_WORDS)
                    .join(" ");
                self.target += &(" ".to_string() + &additional_text);
                self.need_more_lines = false;

                // This might be getting out of hand - stop the
                // unbounded growth!
                if self.target.len() >= TARGET_TEXT_SIZE_LIMIT {
                    self.exit = true;
                    continue;
                }
            }

            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        let source_iter = self.source.split(" ").filter(|&x| x != "");
        let target_iter = self.target.split(" ").filter(|&x| x != "");

        let stats = source_iter.zip(target_iter).fold(
            (0, 0, 0, 0, 0),
            |acc: (usize, usize, usize, usize, usize), (src, trg)| {
                (
                    acc.0.add(if src == trg { 1 } else { 0 }),
                    acc.1.add(
                        src.chars()
                            .take(trg.len())
                            .zip(trg.chars())
                            .filter(|(x, y)| x == y)
                            .count(),
                    ),
                    acc.2.add(
                        src.chars()
                            .take(trg.len())
                            .zip(trg.chars())
                            .filter(|(x, y)| x != y)
                            .count(),
                    ),
                    acc.3.add(src.len().saturating_sub(trg.len())),
                    acc.4.add(trg.len().saturating_sub(src.len())),
                )
            },
        );

        let typing_time = Instant::now()
            .checked_duration_since(self.ends_at.sub(TYPING_TIME))
            .unwrap_or(TYPING_TIME)
            .as_micros();

        Ok((
            (stats.0 * 60000000) as f64 / typing_time as f64,
            "82",
            stats.1,
            stats.2,
            stats.3,
            stats.4,
        ))
    }

    /// Return the line the cursor is at given its index
    /// in the target list of spans
    fn cursor_line_number(
        &self,
        layout_width: u16,
        spans: Vec<Span<'_>>,
        cursor_at: usize,
    ) -> usize {
        let phantom_target = Line::from(&spans[..cursor_at + 1]);
        let phantom_paragraph = Paragraph::new(phantom_target).wrap(Wrap { trim: true });

        phantom_paragraph.line_count(layout_width - PADDING * 2)
    }

    fn draw(&mut self, frame: &mut Frame) {
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
        let mut cursor_at: usize = 0;

        for i in 0..s2.len() {
            if !has_cursor && i == s1.len() {
                s3.push(Span::styled(
                    match s2[i].chars().nth(0) {
                        Some(c) => c.to_string(),
                        None => continue,
                    },
                    Style::default().fg(Color::Black).bg(Color::White),
                ));
                for j in 1..s2.len() {
                    s3.push(Span::styled(s2[j], Style::default().fg(Color::DarkGray)));
                }
                s3.push(Span::styled(" ".to_string(), Style::default()));

                has_cursor = true;
                cursor_at = s3.len() - 1;

                continue;
            } else if i >= s1.len() {
                s3.push(Span::styled(s2[i], Style::default().fg(Color::DarkGray)));
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
                        cursor_at = s3.len() + span_group.len() - 1;
                    } else {
                        span_group.push(Span::styled(
                            target_word[j].to_string(),
                            Style::default().fg(Color::DarkGray),
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
                cursor_at = s3.len() - 1;
            } else {
                s3.push(Span::styled(" ".to_string(), Style::default()))
            }
        }

        let s3_clone = s3.clone();
        let target = Line::from(s3);

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(frame.area());

        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
            .split(layout[0]);

        let paragraph = Paragraph::new(target)
            .block(block.clone())
            .wrap(Wrap { trim: true });

        let cursor_line_number = self.cursor_line_number(layout[0].width, s3_clone, cursor_at);
        let paragraph = paragraph.scroll((cursor_line_number.saturating_sub(2) as u16, 0));
        let lines_left = paragraph
            .line_count(layout[0].width)
            .saturating_sub(cursor_line_number as usize);

        if lines_left <= REQUEST_LINES_THRESHOLD {
            self.need_more_lines = true;
        }

        let debug = Paragraph::new(vec![
            source,
            Line::from(format!(
                "{} {} {} {} {} {:?}",
                paragraph.line_count(layout[0].width - PADDING * 2),
                ((cursor_at / layout[0].width.saturating_sub(PADDING * 2) as usize) as u16),
                layout[0].width - PADDING * 2,
                0, // phantom_paragraph.line_count(block.inner(layout[0]).width),
                block.inner(layout[0]).width,
                paragraph
                    .line_count(layout[0].width)
                    .saturating_sub(cursor_line_number as usize)
            )),
        ])
        .wrap(Wrap { trim: true });

        frame.render_widget(
            Paragraph::new(Line::from(format!(
                " howfast | {}s",
                if !self.started {
                    TYPING_TIME
                } else {
                    self.ends_at.sub(Instant::now())
                }
                .as_secs()
            ))),
            vertical_layout[1],
        );
        frame.render_widget(paragraph, vertical_layout[2]);
        frame.render_widget(debug, layout[1]);
    }

    fn handle_events(&mut self) -> std::io::Result<()> {
        if !event::poll(POLLING_TIME)? {
            return Ok(());
        }

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
            KeyCode::Char(c) => {
                if !self.started {
                    self.started = true;
                    self.ends_at = Instant::now().add(TYPING_TIME)
                }
                self.source.push(c);
            }
            KeyCode::Backspace => {
                self.source.pop();
            }
            _ => {}
        }
    }
}

fn main() -> color_eyre::Result<()> {
    let dictionary = Dictionary::from_file("dictionaries/english_1k.txt")?;
    let mut coordinator = Coordinator::new(dictionary);

    color_eyre::install()?;
    let (wpm, acc, cor, inc, ext, mis) = ratatui::run(|terminal| coordinator.run(terminal))?;

    println!("WPM: {:.2}", wpm);
    println!("Accuracy: {}%", acc);
    println!(
        "{} correct, {} incorrect, {} extra, {} missed",
        cor, inc, ext, mis
    );

    Ok(())
}
