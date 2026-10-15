use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Padding, Paragraph, Wrap};
use ratatui::{DefaultTerminal, Frame};
use std::ops::{Add, Sub};
use std::time::{Duration, Instant};

use crate::Dictionary;

#[derive(Debug, Default)]
pub struct DebugFlags(pub u8);

impl DebugFlags {
    /// Nothing
    pub const NONE: u8 = 0b00;
    /// Render source text with some additional data
    pub const SOURCE_TEXT: u8 = 0b01;
    /// Render text overflow to track text generation
    pub const TEXT_OVERFLOW: u8 = 0b10;
}

impl From<u8> for DebugFlags {
    fn from(value: u8) -> Self {
        DebugFlags(value)
    }
}

#[derive(Debug)]
pub struct Coordinator {
    dictionary: Dictionary,

    source: String,
    target: String,

    typing_time: Duration,
    render_lines: usize,
    /// Request new lines once we reach this render threshold
    render_lines_threshold: usize,
    padding: u16,
    width_percent: u8,
    cursor_scroll_line_offset: usize,
    ends_at: Instant,

    exit: bool,
    started: bool,
    need_more_lines: bool,
    pub debug_flags: DebugFlags,

    // Stats
    correct_keystrokes: usize,
    incorrect_keystrokes: usize,
}

impl Default for Coordinator {
    fn default() -> Self {
        Self {
            dictionary: Default::default(),
            source: Default::default(),
            target: Default::default(),
            typing_time: Default::default(),
            render_lines: Default::default(),
            render_lines_threshold: Default::default(),
            padding: Default::default(),
            width_percent: Default::default(),
            cursor_scroll_line_offset: Default::default(),
            ends_at: Instant::now(),
            exit: Default::default(),
            started: Default::default(),
            need_more_lines: Default::default(),
            debug_flags: Default::default(),
            correct_keystrokes: Default::default(),
            incorrect_keystrokes: Default::default(),
        }
    }
}

impl Coordinator {
    /// How many words to request on initial creation
    /// and future expansions
    const REQUEST_WORDS: usize = 1 << 7;
    /// If the text grows above this threshold, we
    /// should halt the program
    const TARGET_TEXT_SIZE_LIMIT: usize = 1 << 20;
    const POLLING_TIME: Duration = Duration::from_millis(200);

    pub fn new(
        dictionary: Dictionary,
        typing_time_secs: u64,
        render_lines: usize,
        padding: u16,
        width_percent: u8,
        debug_flags: impl Into<DebugFlags>,
    ) -> Self {
        let target = dictionary.get_random_words(Self::REQUEST_WORDS).join(" ");

        Self {
            dictionary,
            source: String::with_capacity(target.len()),
            target,
            typing_time: Duration::from_secs(typing_time_secs),
            render_lines,
            render_lines_threshold: render_lines + 1,
            padding,
            width_percent,
            cursor_scroll_line_offset: render_lines.min(2).saturating_sub(1),
            ends_at: Instant::now(),
            exit: false,
            started: false,
            need_more_lines: false,
            debug_flags: debug_flags.into(),
            correct_keystrokes: 0,
            incorrect_keystrokes: 0,
        }
    }

    pub fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
    ) -> std::io::Result<(f64, f64, usize, usize, usize, usize)> {
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
                if self.target.len() >= Self::TARGET_TEXT_SIZE_LIMIT {
                    self.exit = true;
                    continue;
                }
            }

            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        // Endgame stats
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
            .checked_duration_since(self.ends_at.sub(self.typing_time))
            .unwrap_or(self.typing_time)
            .as_micros();

        Ok((
            stats.0.saturating_mul(60000000) as f64 / typing_time as f64,
            self.correct_keystrokes.saturating_mul(100) as f64
                / (self.correct_keystrokes + self.incorrect_keystrokes).max(1) as f64,
            stats.1,
            stats.2,
            stats.3,
            stats.4,
        ))
    }

    /// Returns the line the cursor is at given its index
    /// in the target list of spans
    fn cursor_line_number(&self, layout_width: u16, spans: &[Span<'_>], cursor_at: usize) -> usize {
        let phantom_decorated_text_line = Line::from(&spans[..cursor_at + 1]);
        let phantom_paragraph =
            Paragraph::new(phantom_decorated_text_line).wrap(Wrap { trim: true });
        phantom_paragraph
            .line_count(layout_width.saturating_sub(self.padding * 2))
            .saturating_sub(1) // 0-based
    }

    /// Returns the next expected correct token based on
    /// the match against source text and target text
    fn expected_correct_token(&self) -> Option<char> {
        if self.target == "" {
            return None;
        }

        let src: Vec<&str> = self.source.split(' ').collect();
        let trg: Vec<&str> = self.target.split(' ').collect();

        if src.len() > trg.len() {
            return None;
        }

        let last_source_word = src[src.len().saturating_sub(1)];
        let target_word = trg[src.len().saturating_sub(1)];

        // If the source is equal or overflowing, then the
        // next token should just be space
        if last_source_word.len() >= target_word.len() {
            return Some(' ');
        }

        target_word.chars().nth(last_source_word.len())
    }

    /// Returns a vector of decorated spans and the position of
    /// the cursor based on the match against source text and
    /// target text
    fn decorated_target_text<'a, 'b>(&'a self) -> (Vec<Span<'b>>, usize) {
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
                    s3.push(Span::styled(
                        s2[j].to_string(),
                        Style::default().fg(Color::DarkGray),
                    ));
                }
                s3.push(Span::styled(" ".to_string(), Style::default()));

                has_cursor = true;
                cursor_at = s3.len() - 1;

                continue;
            } else if i >= s1.len() {
                s3.push(Span::styled(
                    s2[i].to_string(),
                    Style::default().fg(Color::DarkGray),
                ));
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
            };
        }

        (s3, cursor_at)
    }

    fn draw(&mut self, frame: &mut Frame) {
        let horizontal_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(self.width_percent as u16),
                Constraint::Fill(1),
            ])
            .split(frame.area());

        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
            .split(
                Block::default()
                    .borders(Borders::NONE)
                    .padding(Padding::uniform(self.padding))
                    .inner(horizontal_layout[0]),
            );

        let (decorated_text, decorated_text_cursor_at) = self.decorated_target_text();

        let cursor_line_number = self.cursor_line_number(
            horizontal_layout[0].width,
            &decorated_text,
            decorated_text_cursor_at,
        );

        let paragraph = Paragraph::new(Line::from(decorated_text))
            .wrap(Wrap { trim: true })
            .scroll((
                cursor_line_number.saturating_sub(self.cursor_scroll_line_offset) as u16,
                0,
            ));

        let lines_left = paragraph
            .line_count(horizontal_layout[0].width)
            .saturating_sub(cursor_line_number as usize);

        if lines_left <= self.render_lines_threshold {
            self.need_more_lines = true;
        }

        frame.render_widget(
            Paragraph::new(Line::from(format!(
                "howfast | {}s",
                if !self.started {
                    self.typing_time
                } else {
                    self.ends_at.sub(Instant::now())
                }
                .as_secs()
            ))),
            vertical_layout[0],
        );

        let limited_area = Rect::new(
            vertical_layout[2].x,
            vertical_layout[2].y,
            vertical_layout[2].width,
            if self.has_all_flags(DebugFlags::TEXT_OVERFLOW) {
                vertical_layout[2].height
            } else {
                self.render_lines as u16
            },
        );

        frame.render_widget(paragraph, limited_area);

        if self.has_all_flags(DebugFlags::SOURCE_TEXT) {
            let debug = Paragraph::new(vec![
                Line::from(Span::styled(
                    self.source.as_str(),
                    Style::default().fg(Color::White),
                )),
                Line::from(format!(
                    "{} {} {} {} {}",
                    vertical_layout[2].x,
                    vertical_layout[2].y,
                    horizontal_layout[0].width.saturating_sub(self.padding * 2),
                    self.correct_keystrokes,
                    self.incorrect_keystrokes
                )),
            ])
            .wrap(Wrap { trim: true });

            frame.render_widget(debug, horizontal_layout[1]);
        }
    }

    fn handle_events(&mut self) -> std::io::Result<()> {
        if !event::poll(Self::POLLING_TIME)? {
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
                    self.ends_at = Instant::now().add(self.typing_time)
                }

                match self.expected_correct_token() {
                    Some(e) if c == e => {
                        self.correct_keystrokes = self.correct_keystrokes.wrapping_add(1)
                    }
                    _ => self.incorrect_keystrokes = self.incorrect_keystrokes.wrapping_add(1),
                }

                self.source.push(c);
            }
            KeyCode::Backspace => {
                self.source.pop();
            }
            _ => {}
        }
    }

    /// Returns true if the coordinator has all of
    /// the specified flags
    fn has_all_flags(&self, flags: u8) -> bool {
        return self.debug_flags.0 & flags == flags;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expected_correct_token_empty_source_and_target() {
        let mut coordinator = Coordinator::default();

        coordinator.source = "".to_string();
        coordinator.target = "".to_string();

        assert_eq!(coordinator.expected_correct_token(), None);
    }

    #[test]
    fn expected_correct_token_overflowing_source_and_empty_target() {
        let mut coordinator = Coordinator::default();

        coordinator.source = "Why am I overflowing?".to_string();
        coordinator.target = "".to_string();

        assert_eq!(coordinator.expected_correct_token(), None);
    }

    #[test]
    fn expected_correct_token_text_mismatch() {
        let mut coordinator = Coordinator::default();

        coordinator.source = "Why am I overflowing?".to_string();
        coordinator.target = "Because you were supposed to wait.".to_string();

        assert_eq!(coordinator.expected_correct_token(), Some(' '));
    }

    #[test]
    fn expected_correct_token_single_word_matching() {
        let mut coordinator = Coordinator::default();

        coordinator.source = "Why".to_string();
        coordinator.target = "Why am I overflowing?".to_string();

        assert_eq!(coordinator.expected_correct_token(), Some(' '));
    }

    #[test]
    fn expected_correct_token_two_words_almost_matching() {
        let mut coordinator = Coordinator::default();

        coordinator.source = "Why a".to_string();
        coordinator.target = "Why am I overflowing?".to_string();

        assert_eq!(coordinator.expected_correct_token(), Some('m'));
    }

    #[test]
    fn expected_correct_token_three_words_almost_matching() {
        let mut coordinator = Coordinator::default();

        coordinator.source = "Why am ".to_string();
        coordinator.target = "Why am I overflowing?".to_string();

        assert_eq!(coordinator.expected_correct_token(), Some('I'));
    }

    #[test]
    fn expected_correct_token_bad_text_start() {
        let mut coordinator = Coordinator::default();

        coordinator.source = "Bad te x t".to_string();
        coordinator.target = "Why am I overflowing?".to_string();

        assert_eq!(coordinator.expected_correct_token(), Some('v'));
    }

    #[test]
    fn expected_correct_token_bad_text_start_longer() {
        let mut coordinator = Coordinator::default();

        coordinator.source = "Bad te x tovbfl".to_string();
        coordinator.target = "Why am I overflOwing?".to_string();

        assert_eq!(coordinator.expected_correct_token(), Some('O'));
    }
}
