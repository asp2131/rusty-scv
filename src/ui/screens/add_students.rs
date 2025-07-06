use anyhow::Result;
use crossterm::event::{KeyEvent, KeyCode};
use ratatui::{
    layout::{Alignment, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use std::pin::Pin;
use tokio::pin;

use crate::{
    app::AppEvent,
    data::{Class, Database},
    ui::themes::Theme,
};

pub struct AddStudentsScreen {
    class: Class,
    input_text: String,
}

impl AddStudentsScreen {
    pub fn new(class: Class) -> Self {
        Self {
            class,
            input_text: String::new(),
        }
    }
}

impl super::Screen for AddStudentsScreen {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn screen_type(&self) -> super::ScreenType {
        super::ScreenType::new(super::ScreenTypeVariant::AddStudents)
            .with_context(super::ScreenContext::Class(self.class.clone()))
    }

    fn update<'a>(&'a mut self, _delta_time: std::time::Duration, _state: &'a mut crate::app::AppState) -> Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move { Ok(()) })
    }

    fn handle_key_event(&mut self, key: KeyEvent, state: &crate::app::AppState) -> Pin<Box<dyn std::future::Future<Output = Result<Option<AppEvent>>> + Send + '_>> {
        match key.code {
            KeyCode::Esc => Box::pin(async move { Ok(Some(AppEvent::GoBack)) }),
            KeyCode::Char(c) => {
                self.input_text.push(c);
                Box::pin(async move { Ok(None) })
            }
            KeyCode::Backspace => {
                self.input_text.pop();
                Box::pin(async move { Ok(None) })
            }
            KeyCode::Enter => {
                if self.input_text.trim().is_empty() {
                    return Box::pin(async move { Ok(None) });
                }
                
                let students = self.input_text.split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .map(|name| (self.class.id, name.to_string()))
                    .collect::<Vec<_>>();
                
                if students.is_empty() {
                    return Box::pin(async move { Ok(None) });
                }
                
                let db = state.database.clone();
                let input_text = std::mem::take(&mut self.input_text);
                
                Box::pin(async move {
                    // Add each student to database
                    for (class_id, username) in students {
                        if let Err(e) = db.add_student(class_id, &username).await {
                            // TODO: Show error to user
                            log::error!("Failed to add student {}: {}", username, e);
                        }
                    }
                    
                    Ok(Some(AppEvent::GoBack))
                })
            }
            _ => Box::pin(async move { Ok(None) }),
        }
    }

    fn render(
        &mut self, 
        frame: &mut ratatui::Frame<ratatui::backend::CrosstermBackend<std::io::Stdout>>, 
        area: Rect, 
        state: &crate::app::AppState, 
        _animation_state: &crate::ui::animations::AnimationState, 
        theme: &Theme
    ) {
        frame.render_widget(Clear, area);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!("Add Students to: {}", self.class.name))
            .border_style(Style::default().fg(theme.primary));

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        let instruction = Paragraph::new(vec![Line::from(Span::styled(
            "Enter student names (comma separated):", 
            Style::default().fg(theme.text)
        ))]);

        let input = Paragraph::new(self.input_text.as_str())
            .style(Style::default().fg(theme.text))
            .block(Block::default().borders(Borders::ALL).title("Input"));

        frame.render_widget(instruction, Rect {
            x: inner_area.x,
            y: inner_area.y,
            width: inner_area.width,
            height: 1,
        });
        frame.render_widget(input, Rect {
            x: inner_area.x,
            y: inner_area.y + 2,
            width: inner_area.width,
            height: 3,
        });

        let help_text = Paragraph::new(vec![Line::from(vec![
            Span::styled("esc back • ", Style::default().fg(theme.text_secondary)),
            Span::styled("enter submit", Style::default().fg(theme.text_secondary)),
        ])])
        .alignment(Alignment::Center);

        let help_area = Rect {
            x: inner_area.x,
            y: inner_area.y + inner_area.height.saturating_sub(1),
            width: inner_area.width,
            height: 1,
        };

        frame.render_widget(help_text, help_area);
    }
}
