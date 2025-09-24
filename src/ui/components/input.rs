use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};
use crossterm::event::{KeyCode, KeyEvent};
use std::time::Duration;



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
    
    pub fn insert_text(&mut self, text: &str) {
        // Optimized bulk text insertion
        let char_count = self.value.chars().count();
        
        if self.cursor_position >= char_count {
            // Append to end - most efficient for paste operations
            self.value.push_str(text);
        } else {
            // Insert at specific position
            let char_indices: Vec<_> = self.value.char_indices().collect();
            let byte_pos = if self.cursor_position >= char_indices.len() {
                self.value.len()
            } else {
                char_indices[self.cursor_position].0
            };
            self.value.insert_str(byte_pos, text);
        }
        
        self.cursor_position += text.chars().count();
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
                // Optimized character insertion - avoid recreating char_indices for every character
                if self.cursor_position >= char_count {
                    // Append to end - most common case for paste operations
                    self.value.push(c);
                } else {
                    // Insert at specific position - only collect indices when needed
                    let char_indices: Vec<_> = self.value.char_indices().collect();
                    let byte_pos = if self.cursor_position >= char_indices.len() {
                        self.value.len()
                    } else {
                        char_indices[self.cursor_position].0
                    };
                    self.value.insert(byte_pos, c);
                }
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

impl Widget for &mut AnimatedInput {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Theme should be passed from the parent context
        // For now, we'll use the default theme but this should be improved
        let theme = &crate::ui::themes::THEMES.neon_night;
        
        let border_style = if self.focused {
            theme.border_focused_style()
        } else {
            theme.border_style()
        };
        
        let title = Line::from(Span::styled(
            self.title.as_str(),
            theme.primary_text()
        ));
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style);
        
        let inner_area = block.inner(area);
        block.render(area, buf);
        
        let available_width = inner_area.width as usize;
        if available_width < 2 {
            return;
        }
        
        self.ensure_cursor_visible(available_width);
        
        // Get the visible portion of the text
        let chars: Vec<char> = self.value.chars().collect();
        let end_pos = std::cmp::min(
            self.scroll_offset + available_width,
            chars.len()
        );
        
        let visible_text: String = if self.scroll_offset < chars.len() {
            chars[self.scroll_offset..end_pos].iter().collect()
        } else {
            String::new()
        };
        
        // Prepare the display text
        let display_text = if self.value.is_empty() && !self.placeholder.is_empty() {
            self.placeholder.as_str()
        } else {
            &visible_text
        };
        
        let text_style = if self.value.is_empty() && !self.placeholder.is_empty() {
            theme.secondary_text()
        } else {
            theme.primary_text()
        };
        
        // Add cursor if focused
        let line = if self.focused {
            let cursor_visible = self.cursor_blink.sin() > 0.0;
            // Use a bright white color for maximum visibility
            let focused_text_style = Style::default().fg(Color::White);
            
            if self.value.is_empty() {
                // Empty input - show cursor with placeholder
                if cursor_visible {
                    if !self.placeholder.is_empty() {
                        Line::from(vec![
                            Span::styled("█", Style::default().fg(Color::Cyan)),
                            Span::styled(&self.placeholder, theme.secondary_text()),
                        ])
                    } else {
                        Line::from(Span::styled("█", Style::default().fg(Color::Cyan)))
                    }
                } else {
                    // Cursor not visible - show just placeholder or empty
                    if !self.placeholder.is_empty() {
                        Line::from(Span::styled(&self.placeholder, theme.secondary_text()))
                    } else {
                        Line::from(Span::styled(" ", focused_text_style)) // Empty space to maintain height
                    }
                }
            } else {
                // Input has text - show text with cursor
                let visible_cursor_pos = self.cursor_position.saturating_sub(self.scroll_offset);
                
                // Debug: Make sure we're actually showing the text
                if visible_text.is_empty() && !self.value.is_empty() {
                    // If value has text but visible_text is empty, something's wrong
                    Line::from(Span::styled(&self.value, focused_text_style))
                } else if visible_cursor_pos >= visible_text.chars().count() {
                    // Cursor at end of visible text
                    if cursor_visible {
                        Line::from(vec![
                            Span::styled(&visible_text, focused_text_style),
                            Span::styled("█", Style::default().fg(Color::Cyan)),
                        ])
                    } else {
                        Line::from(Span::styled(&visible_text, focused_text_style))
                    }
                } else {
                    // Cursor in middle of visible text
                    let visible_chars: Vec<char> = visible_text.chars().collect();
                    let before: String = visible_chars[..visible_cursor_pos].iter().collect();
                    let after: String = visible_chars[visible_cursor_pos..].iter().collect();
                    
                    if cursor_visible {
                        Line::from(vec![
                            Span::styled(before, focused_text_style),
                            Span::styled("|", Style::default().fg(Color::Cyan)),
                            Span::styled(after, focused_text_style),
                        ])
                    } else {
                        Line::from(Span::styled(&visible_text, focused_text_style))
                    }
                }
            }
        } else {
            // Not focused - show text or placeholder without cursor
            Line::from(Span::styled(display_text, text_style))
        };
        
        let paragraph = Paragraph::new(line);
        paragraph.render(inner_area, buf);
    }
}