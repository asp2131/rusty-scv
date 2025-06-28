use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame, 
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};
use std::time::Duration;

use crate::{
    app::{AppEvent, AppState},
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

    fn handle_key_event(&mut self, key: KeyEvent, state: &AppState) -> Pin<Box<dyn Future<Output = Result<Option<AppEvent>>> + Send + '_>> {
        if self.creating {
            return Box::pin(async move { Ok(None) }); // Ignore input while creating
        }
        
        let result = match key.code {
            KeyCode::Enter => {
                let class_name = self.input.value().trim();
                if class_name.is_empty() {
                    self.error = Some("Class name cannot be empty".to_string());
                    Ok(None)
                } else {
                    // TODO: Implement async class creation
                    Ok(Some(AppEvent::ShowSuccess(format!("Created class: {}", class_name))))
                }
            },
            KeyCode::Esc => {
                Ok(Some(AppEvent::GoBack))
            },
            _ => {
                self.input.handle_key_event(key);
                self.error = None; // Clear error on new input
                Ok(None)
            }
        };
        Box::pin(async move { result })
    }

    fn update(&mut self, delta_time: Duration, _state: &mut AppState) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        self.input.update(delta_time);
        Box::pin(async move { Ok(()) })
    }

    fn render(&mut self, frame: &mut ratatui::Frame<ratatui::backend::CrosstermBackend<std::io::Stdout>>, area: Rect, _state: &AppState, _animation_state: &AnimationState, theme: &Theme) {
        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Length(3),  // Input
                Constraint::Length(2),  // Error/Help
                Constraint::Min(0),     // Remaining
            ])
            .split(area);

        // Title
        let title = Paragraph::new("📚 Create New Class")
            .style(theme.primary_text())
            .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // Input field
        frame.render_widget(&mut self.input, chunks[1]);

        // Error or help text
        if let Some(ref error) = self.error {
            let error_text = Paragraph::new(error.as_str())
                .style(theme.error_text())
                .alignment(Alignment::Center);
            frame.render_widget(error_text, chunks[2]);
        } else if !self.creating {
            let help_text = Paragraph::new("Press Enter to create • Esc to cancel")
                .style(theme.secondary_text())
                .alignment(Alignment::Center);
            frame.render_widget(help_text, chunks[2]);
        }

        // Loading overlay
        if self.creating {
            let loading_area = crate::ui::layout::center_rect(40, 20, area);
            frame.render_widget(Clear, loading_area);
            
            let loading = crate::ui::components::loading::LoadingPresets::initializing(theme);
            frame.render_widget(loading, loading_area);
        }
    }
}