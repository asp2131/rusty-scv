use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};
use std::{future::Future, pin::Pin, time::Duration};

use crate::{
    app::{AppEvent, AppState},
    data::Class,
    ui::{
        animations::AnimationState,
        components::menu::{AnimatedMenu, MenuBuilder, MenuItem, MenuPresets},
        screens::{Screen, ScreenType, ScreenTypeVariant},
        themes::Theme,
    },
};

pub struct ClassManagementScreen {
    class: Class,
    menu: AnimatedMenu,
}

impl ClassManagementScreen {
    pub fn new(class: Class) -> Self {
        let menu = MenuPresets::class_management(&class.name);
        
        Self { 
            class, 
            menu
        }
    }
}

impl Screen for ClassManagementScreen {
    fn screen_type(&self) -> ScreenType {
        ScreenType::new(ScreenTypeVariant::ClassManagement)
    }

    fn handle_key_event<'a>(
        &'a mut self,
        key: KeyEvent,
        _state: &'a AppState,
    ) -> Pin<Box<dyn Future<Output = Result<Option<AppEvent>>> + Send + 'a>> {
        let result = match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.menu.select_previous();
                Ok(None)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.menu.select_next();
                Ok(None)
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                if let Some(selected) = self.menu.selected_item() {
                    match selected.title.as_str() {
                        "Manage Students" => Ok(Some(AppEvent::NavigateToScreen(
                            ScreenType::new(ScreenTypeVariant::StudentManagement)
                        ))),
                        "Manage Repos" => Ok(Some(AppEvent::NavigateToScreen(
                            ScreenType::new(ScreenTypeVariant::RepositoryManagement)
                        ))),
                        "View GH Activity" => Ok(Some(AppEvent::NavigateToScreen(
                            ScreenType::new(ScreenTypeVariant::GitHubActivity)
                        ))),
                        "Delete Class" => Ok(Some(AppEvent::NavigateToScreen(
                            ScreenType::new(ScreenTypeVariant::ConfirmDeleteClass)
                        ))),
                        "Back" => Ok(Some(AppEvent::GoBack)),
                        _ => Ok(None),
                    }
                } else {
                    Ok(None)
                }
            }
            KeyCode::Esc => Ok(Some(AppEvent::GoBack)),
            _ => Ok(None),
        };

        Box::pin(async { result })
    }

    fn update<'a>(
        &'a mut self,
        delta_time: Duration,
        _state: &'a mut AppState,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        // Update menu animations
        let animation_state = AnimationState::default();
        self.menu.update(delta_time, &animation_state);
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
        let popup_area = crate::ui::layout::center_rect(70, 40, area);
        
        // Create a block for the content
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Class Management")
            .title_alignment(Alignment::Center)
            .style(Style::default().bg(theme.background).fg(theme.text));
            
        frame.render_widget(block, popup_area);
        
        // Create inner area for content
        let inner_area = popup_area.inner(&crate::ui::layout::margin(1, 1));
        
        // Create layout for the menu
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(1),    // Menu
                Constraint::Length(1), // Help text
            ])
            .split(inner_area);
        
        // Render title
        let title = Paragraph::new(format!("Managing Class: {}", self.class.name))
            .alignment(Alignment::Center)
            .style(Style::default().add_modifier(Modifier::BOLD));
        frame.render_widget(title, chunks[0]);
        
        // Render the menu
        frame.render_widget(&mut self.menu, chunks[1]);
        
        // Render help text
        let help_text = Line::from(vec![
            Span::styled("↑/↓", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": Navigate  "),
            Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": Select  "),
            Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": Back"),
        ]);
        
        frame.render_widget(
            Paragraph::new(help_text)
                .alignment(Alignment::Center)
                .style(Style::default().fg(theme.text_secondary)),
            chunks[2],
        );
    }
}
