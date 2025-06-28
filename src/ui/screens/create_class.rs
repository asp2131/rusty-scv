use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame, 
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
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
        if let KeyCode::Esc = key.code {
            return Box::pin(async { Ok(Some(AppEvent::GoBack)) });
        }

        // Always handle input events to allow typing
        self.input.handle_key_event(key);
        
        if let KeyCode::Enter = key.code {
            let class_name = self.input.get_text().to_string();
            if class_name.is_empty() {
                self.error = Some("Class name cannot be empty".to_string());
            } else {
                // Create a new Class with the provided name
                let class = Class {
                    id: 0, // Will be set by the database
                    name: class_name,
                    created_at: chrono::Utc::now(),
                };
                self.error = None; // Clear any previous errors
                return Box::pin(async { Ok(Some(AppEvent::ClassCreated(class))) });
            }
        } else {
            // Clear error on new input
            self.error = None;
        }
        
        Box::pin(async { Ok(None) })
    }

    fn update<'a>(
        &'a mut self,
        _delta_time: Duration,
        _state: &'a mut AppState,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async { Ok(()) })
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
        
        // Create a block for the content
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Create New Class")
            .title_alignment(Alignment::Center)
            .style(Style::default().bg(theme.background).fg(theme.text));
            
        frame.render_widget(block, popup_area);
        
        // Create inner area for content
        let inner_area = popup_area.inner(&crate::ui::layout::margin(1, 1));
        
        // Create layout for the form
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(3), // Input field
                Constraint::Min(1),    // Error message
                Constraint::Length(1), // Help text
            ])
            .split(inner_area);
        
        // Render title
        let title = Paragraph::new("Create New Class")
            .alignment(Alignment::Center)
            .style(Style::default().add_modifier(Modifier::BOLD));
        frame.render_widget(title, chunks[0]);
        
        // Render input field
        let input_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default()
                .bg(theme.background)
                .fg(theme.text));
                
        let input_text = if self.input.get_text().is_empty() {
            Line::from(Span::styled(
                "Enter class name",
                Style::default().fg(theme.text_secondary),
            ))
        } else {
            Line::from(Span::styled(
                self.input.get_text(),
                Style::default().fg(theme.text),
            ))
        };
        
        frame.render_widget(
            Paragraph::new(input_text)
                .block(input_block)
                .alignment(Alignment::Left),
            chunks[1],
        );
        
        // Render cursor if focused
        if self.input.is_focused() {
            let cursor_x = self.input.cursor_position() as u16 + 1; // +1 for border
            frame.set_cursor(
                popup_area.x + cursor_x + 1, // +1 for left border
                popup_area.y + 2, // +2 for title and top border
            );
        }
        
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
        let help_text = Line::from(vec![
            Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": Create Class  "),
            Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": Back"),
        ]);
        
        frame.render_widget(
            Paragraph::new(help_text)
                .alignment(Alignment::Center)
                .style(Style::default().fg(theme.text_secondary)),
            chunks[3],
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