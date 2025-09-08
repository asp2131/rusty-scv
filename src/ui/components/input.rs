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
    scroll_offset: usize,
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
            scroll_offset: 0,
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
        self.cursor_position = self.value.chars().count();
        self.scroll_offset = 0;
    }
    
    fn ensure_cursor_visible(&mut self, available_width: usize) {
        if available_width == 0 {
            return;
        }
        
        let char_count = self.value.chars().count();
        
        // Ensure cursor_position is valid
        if self.cursor_position > char_count {
            self.cursor_position = char_count;
        }
        
        // Adjust scroll to keep cursor visible
        if self.cursor_position < self.scroll_offset {
            self.scroll_offset = self.cursor_position;
        } else if self.cursor_position >= self.scroll_offset + available_width {
            self.scroll_offset = self.cursor_position.saturating_sub(available_width - 1);
        }
    }
    
    pub fn handle_key_event(&mut self, key: KeyEvent) {
        let char_count = self.value.chars().count();
        
        match key.code {
            KeyCode::Char(c) => {
                // Insert character at cursor position using char indices
                let char_indices: Vec<_> = self.value.char_indices().collect();
                let byte_pos = if self.cursor_position >= char_indices.len() {
                    self.value.len()
                } else {
                    char_indices[self.cursor_position].0
                };
                self.value.insert(byte_pos, c);
                self.cursor_position += 1;
            },
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    let char_indices: Vec<_> = self.value.char_indices().collect();
                    if self.cursor_position < char_indices.len() {
                        let byte_pos = char_indices[self.cursor_position].0;
                        self.value.remove(byte_pos);
                    }
                }
            },
            KeyCode::Delete => {
                let char_indices: Vec<_> = self.value.char_indices().collect();
                if self.cursor_position < char_indices.len() {
                    let byte_pos = char_indices[self.cursor_position].0;
                    self.value.remove(byte_pos);
                }
            },
            KeyCode::Left => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
            },
            KeyCode::Right => {
                if self.cursor_position < char_count {
                    self.cursor_position += 1;
                }
            },
            KeyCode::Home => {
                self.cursor_position = 0;
            },
            KeyCode::End => {
                self.cursor_position = char_count;
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
        
        let available_width = inner_area.width as usize;
        
        // Create a mutable copy to adjust scrolling
        let mut input_copy = AnimatedInput {
            value: self.value.clone(),
            placeholder: self.placeholder.clone(),
            title: self.title.clone(),
            focused: self.focused,
            cursor_position: self.cursor_position,
            cursor_blink: self.cursor_blink,
            scroll_offset: self.scroll_offset,
        };
        
        input_copy.ensure_cursor_visible(available_width);
        
        // Get the visible portion of the text
        let chars: Vec<char> = input_copy.value.chars().collect();
        let end_pos = std::cmp::min(
            input_copy.scroll_offset + available_width,
            chars.len()
        );
        
        let visible_text: String = if input_copy.scroll_offset < chars.len() {
            chars[input_copy.scroll_offset..end_pos].iter().collect()
        } else {
            String::new()
        };
        
        // Prepare the display text
        let display_text = if input_copy.value.is_empty() && !input_copy.placeholder.is_empty() {
            input_copy.placeholder.as_str()
        } else {
            &visible_text
        };
        
        let text_style = if input_copy.value.is_empty() && !input_copy.placeholder.is_empty() {
            theme.secondary_text()
        } else {
            Style::default().fg(Color::White)
        };
        
        // Add cursor if focused
        let line = if input_copy.focused && !input_copy.value.is_empty() {
            let cursor_visible = input_copy.cursor_blink.sin() > 0.0;
            let white_style = Style::default().fg(Color::White);
            
            // Calculate cursor position relative to visible text
            let visible_cursor_pos = input_copy.cursor_position.saturating_sub(input_copy.scroll_offset);
            
            if visible_cursor_pos >= visible_text.chars().count() {
                // Cursor at end of visible text
                if cursor_visible {
                    Line::from(vec![
                        Span::styled(&visible_text, white_style),
                        Span::styled("█", Style::default().fg(Color::Cyan)),
                    ])
                } else {
                    Line::from(Span::styled(&visible_text, white_style))
                }
            } else {
                // Cursor in middle of visible text
                let visible_chars: Vec<char> = visible_text.chars().collect();
                let before: String = visible_chars[..visible_cursor_pos].iter().collect();
                let after: String = visible_chars[visible_cursor_pos..].iter().collect();
                
                if cursor_visible {
                    Line::from(vec![
                        Span::styled(before, white_style),
                        Span::styled("|", Style::default().fg(Color::Cyan)),
                        Span::styled(after, white_style),
                    ])
                } else {
                    Line::from(Span::styled(&visible_text, white_style))
                }
            }
        } else if input_copy.focused && input_copy.value.is_empty() {
            // Show cursor for empty focused input
            let cursor_visible = input_copy.cursor_blink.sin() > 0.0;
            if cursor_visible {
                Line::from(vec![
                    Span::styled(&input_copy.placeholder, theme.secondary_text()),
                    Span::styled("█", Style::default().fg(Color::Cyan)),
                ])
            } else {
                Line::from(Span::styled(&input_copy.placeholder, theme.secondary_text()))
            }
        } else {
            Line::from(Span::styled(display_text, text_style))
        };
        
        let paragraph = Paragraph::new(line);
        paragraph.render(inner_area, buf);
    }
}