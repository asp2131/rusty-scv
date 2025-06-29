use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame, 
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};
use std::{
    future::Future,
    pin::Pin,
    time::Duration,
};

use crate::{
    app::{AppEvent, AppState},
    data::Class,
    ui::{
        animations::AnimationState,
        components::input::AnimatedInput,
        screens::{Screen, ScreenType},
        themes::Theme,
    },
};

pub struct CreateClassScreen {
    input: AnimatedInput,
    error: Option<String>,
    creating: bool,
}

impl CreateClassScreen {
    pub fn new() -> Self {
        let mut input = AnimatedInput::new("Class Name");
        input.set_placeholder("Enter class name (e.g., 'CS101 Fall 2024')");
        input.focus();
        
        Self {
            input,
            error: None,
            creating: false,
        }
    }
}

impl Screen for CreateClassScreen {
    fn screen_type(&self) -> ScreenType {
        ScreenType::CreateClass
    }

    fn handle_key_event<'a>(
        &'a mut self, 
        key: KeyEvent, 
        _state: &'a AppState,
    ) -> Pin<Box<dyn Future<Output = Result<Option<AppEvent>>> + Send + 'a>> {
        if self.creating {
            return Box::pin(async { Ok(None) });
        }

        match key.code {
            KeyCode::Esc => {
                return Box::pin(async { Ok(Some(AppEvent::GoBack)) });
            }
            KeyCode::Enter => {
                let class_name = self.input.value().trim().to_string();
                if class_name.is_empty() {
                    self.error = Some("Class name cannot be empty".to_string());
                } else {
                    self.creating = true;
                    self.error = None;
                    return Box::pin(async move { 
                        Ok(Some(AppEvent::ShowLoading(format!("Creating class '{}'...", class_name))))
                    });
                }
            }
            _ => {
                // Handle input for typing
                self.input.handle_key_event(key);
                self.error = None; // Clear error on new input
            }
        }
        
        Box::pin(async { Ok(None) })
    }

    fn update<'a>(
        &'a mut self,
        delta_time: Duration,
        _state: &'a mut AppState,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        self.input.update(delta_time);
        Box::pin(async { Ok(()) })
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn render(
        &mut self,
        frame: &mut Frame<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
        area: Rect,
        _state: &AppState,
        _animation_state: &AnimationState,
        theme: &Theme,
    ) {
        // Create a centered area for the content
        let popup_area = crate::ui::layout::center_rect(60, 30, area);
        
        // Clear the area first
        frame.render_widget(Clear, popup_area);
        
        // Create a block for the content
        let block = Block::default()
            .borders(Borders::ALL)
            .title("📚 Create New Class")
            .title_alignment(Alignment::Center)
            .style(Style::default().bg(theme.background).fg(theme.text));
            
        frame.render_widget(block, popup_area);
        
        // Create inner area for content
        let inner_area = popup_area.inner(&crate::ui::layout::margin(1, 1));
        
        // Create layout for the form
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Title
                Constraint::Length(3), // Input field
                Constraint::Length(2), // Error message
                Constraint::Min(1),    // Spacing
                Constraint::Length(2), // Help text
            ])
            .split(inner_area);
        
        // Render title
        let title = Paragraph::new("Enter a name for your new class")
            .alignment(Alignment::Center)
            .style(Style::default().fg(theme.text));
        frame.render_widget(title, chunks[0]);
        
        // Render the input component
        frame.render_widget(&self.input, chunks[1]);
        
        // Render error message if any
        if let Some(error) = &self.error {
            let error_text = Paragraph::new(Line::from(Span::styled(
                error,
                Style::default().fg(theme.error),
            )))
            .alignment(Alignment::Center);
            
            frame.render_widget(error_text, chunks[2]);
        }
        
        // Render help text
        let help_text = if self.creating {
            Line::from(Span::styled(
                "Creating class...",
                Style::default().fg(theme.text_secondary).add_modifier(Modifier::ITALIC),
            ))
        } else {
            Line::from(vec![
                Span::styled("Enter", Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
                Span::styled(": Create  ", Style::default().fg(theme.text_secondary)),
                Span::styled("Esc", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
                Span::styled(": Cancel", Style::default().fg(theme.text_secondary)),
            ])
        };
        
        frame.render_widget(
            Paragraph::new(help_text)
                .alignment(Alignment::Center),
            chunks[4],
        );

        // Loading overlay
        if self.creating {
            let loading_area = crate::ui::layout::center_rect(40, 20, area);
            frame.render_widget(Clear, loading_area);
            
            let loading = crate::ui::components::loading::LoadingPresets::initializing(theme);
            frame.render_widget(loading, loading_area);
        }
    }
}