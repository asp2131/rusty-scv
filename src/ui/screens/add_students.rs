use anyhow::Result;
use crossterm::event::{KeyEvent, KeyCode};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};
use std::pin::Pin;

use crate::{app::AppEvent, data::Class, ui::{components::input::AnimatedInput, themes::Theme}};

pub struct AddStudentsScreen {
    class: Class,
    input: AnimatedInput,
}

impl AddStudentsScreen {
    pub fn new(class: Class) -> Self {
        let mut input = AnimatedInput::new("GitHub Usernames");
        input.set_placeholder("Enter GitHub usernames separated by commas (e.g., user1, user2, user3)");
        input.focus();
        
        Self {
            class,
            input,
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

    fn update<'a>(&'a mut self, delta_time: std::time::Duration, _state: &'a mut crate::app::AppState) -> Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        self.input.update(delta_time);
        Box::pin(async move { Ok(()) })
    }

    fn handle_key_event(&mut self, key: KeyEvent, state: &crate::app::AppState) -> Pin<Box<dyn std::future::Future<Output = Result<Option<AppEvent>>> + Send + '_>> {
        match key.code {
            KeyCode::Esc => Box::pin(async move { Ok(Some(AppEvent::GoBack)) }),
            KeyCode::Enter => {
                let input_text = self.input.value().trim();
                if input_text.is_empty() {
                    return Box::pin(async move { Ok(None) });
                }
                
                // Parse and validate usernames
                let usernames: Vec<String> = input_text.split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .filter(|s| s.len() <= 39) // GitHub username max length
                    .filter(|s| s.chars().all(|c| c.is_alphanumeric() || c == '-'))
                    .map(|s| s.to_string())
                    .collect();
                
                if usernames.is_empty() {
                    return Box::pin(async move { Ok(None) });
                }
                
                let class_id = self.class.id;
                let db = state.database.clone();
                
                Box::pin(async move {
                    let mut successful_adds = 0;
                    let mut failed_adds = 0;
                    
                    // Process students in smaller batches to avoid overwhelming the database
                    for batch in usernames.chunks(10) {
                        for username in batch {
                            match db.add_student(class_id, username).await {
                                Ok(_) => {
                                    successful_adds += 1;
                                    log::info!("Successfully added student: {}", username);
                                }
                                Err(e) => {
                                    failed_adds += 1;
                                    log::warn!("Failed to add student {}: {}", username, e);
                                    // Continue processing other students instead of failing completely
                                }
                            }
                        }
                        
                        // Small delay between batches to prevent overwhelming the system
                        if usernames.len() > 10 {
                            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                        }
                    }
                    
                    log::info!("Bulk add completed: {} successful, {} failed", successful_adds, failed_adds);
                    Ok(Some(AppEvent::GoBack))
                })
            }
            _ => {
                // Handle all other input through the AnimatedInput component
                self.input.handle_key_event(key);
                Box::pin(async move { Ok(None) })
            }
        }
    }

    fn render(
        &mut self, 
        frame: &mut ratatui::Frame<ratatui::backend::CrosstermBackend<std::io::Stdout>>, 
        area: Rect, 
        _state: &crate::app::AppState, 
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

        // Create layout for the form
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Instruction
                Constraint::Length(3), // Input field
                Constraint::Min(1),    // Spacing
                Constraint::Length(2), // Help text
            ])
            .split(inner_area);

        // Render instruction
        let instruction = Paragraph::new(vec![Line::from(Span::styled(
            "Enter GitHub usernames separated by commas:", 
            Style::default().fg(theme.text)
        ))])
        .alignment(Alignment::Center);
        frame.render_widget(instruction, chunks[0]);

        // Render the animated input component
        frame.render_widget(&mut self.input, chunks[1]);

        // Render help text
        let help_text = Paragraph::new(vec![Line::from(vec![
            Span::styled("Esc", Style::default().fg(theme.warning)),
            Span::styled(": Back  ", Style::default().fg(theme.text_secondary)),
            Span::styled("Enter", Style::default().fg(theme.success)),
            Span::styled(": Add Students", Style::default().fg(theme.text_secondary)),
        ])])
        .alignment(Alignment::Center);

        frame.render_widget(help_text, chunks[3]);
    }
}
