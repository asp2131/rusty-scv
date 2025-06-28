use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};
use crossterm::event::{KeyCode, KeyEvent};
use std::time::Duration;

use crate::ui::{animations::AnimationState, themes::Theme};

pub struct AnimatedInput {
    value: String,
    placeholder: String,
    title: String,
    focused: bool,
    cursor_position: usize,
    cursor_blink: f32,
}

impl AnimatedInput {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            value: String::new(),
            placeholder: String::new(),
            title: title.into(),
            focused: false,
            cursor_position: 0,
            cursor_blink: 0.0,
        }
    }
    
    pub fn set_placeholder(&mut self, placeholder: impl Into<String>) {
        self.placeholder = placeholder.into();
    }
    
    pub fn focus(&mut self) {
        self.focused = true;
    }
    
    pub fn unfocus(&mut self) {
        self.focused = false;
    }
    
    pub fn value(&self) -> &str {
        &self.value
    }
    
    pub fn get_text(&self) -> &str {
        &self.value
    }
    
    pub fn is_focused(&self) -> bool {
        self.focused
    }
    
    pub fn cursor_position(&self) -> usize {
        self.cursor_position
    }
    
    pub fn set_value(&mut self, value: String) {
        self.value = value;
        self.cursor_position = self.value.len();
    }
    
    pub fn handle_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(c) => {
                self.value.insert(self.cursor_position, c);
                self.cursor_position += 1;
            },
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    self.value.remove(self.cursor_position);
                }
            },
            KeyCode::Delete => {
                if self.cursor_position < self.value.len() {
                    self.value.remove(self.cursor_position);
                }
            },
            KeyCode::Left => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
            },
            KeyCode::Right => {
                if self.cursor_position < self.value.len() {
                    self.cursor_position += 1;
                }
            },
            KeyCode::Home => {
                self.cursor_position = 0;
            },
            KeyCode::End => {
                self.cursor_position = self.value.len();
            },
            _ => {}
        }
    }
    
    pub fn update(&mut self, delta_time: Duration) {
        if self.focused {
            self.cursor_blink += delta_time.as_secs_f32() * 2.0; // 2Hz blink rate
        }
    }
}

impl Widget for &AnimatedInput {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let theme = &crate::ui::themes::THEMES.neon_night; // TODO: Get from context
        
        let border_style = if self.focused {
            theme.border_focused_style()
        } else {
            theme.border_style()
        };
        
        let block = Block::default()
            .title(self.title.as_str())
            .borders(Borders::ALL)
            .border_style(border_style)
            .title_style(theme.primary_text());
        
        let inner_area = block.inner(area);
        block.render(area, buf);
        
        // Prepare the display text
        let display_text = if self.value.is_empty() && !self.placeholder.is_empty() {
            self.placeholder.as_str()
        } else {
            self.value.as_str()
        };
        
        let text_style = if self.value.is_empty() && !self.placeholder.is_empty() {
            theme.secondary_text()
        } else {
            Style::default().fg(theme.text)
        };
        
        // Add cursor if focused
        let line = if self.focused && self.cursor_blink.sin() > 0.0 {
            let cursor_char = if self.cursor_position >= self.value.len() { "█" } else { "|" };
            let (before, after) = self.value.split_at(self.cursor_position);
            Line::from(vec![
                Span::styled(before, text_style),
                Span::styled(cursor_char, theme.primary_text()),
                Span::styled(after, text_style),
            ])
        } else {
            Line::from(Span::styled(display_text, text_style))
        };
        
        let paragraph = Paragraph::new(line);
        paragraph.render(inner_area, buf);
    }
}