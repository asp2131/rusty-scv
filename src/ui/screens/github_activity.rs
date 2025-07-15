use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::{future::Future, pin::Pin, time::Duration};

use crate::{ui::screens::ScreenContext,
    app::{AppEvent, AppState},
    data::Class,
    ui::{
        animations::AnimationState,
        components::menu::{AnimatedMenu, MenuBuilder, MenuItem, MenuPresets},
        screens::{Screen, ScreenType, ScreenTypeVariant},
        themes::Theme,
    },
};

pub struct GitHubActivityScreen {
    class: Class,
    menu: AnimatedMenu,
}

impl GitHubActivityScreen {
    pub fn new(class: Class) -> Self {
        let menu = MenuBuilder::new()
            .title(&format!("GitHub Activity for Class: {}", class.name))
            .item(MenuItem::new("Week View")
                .with_description("View student activity for the past week")
                .with_icon("🗓️"))
            .item(MenuItem::new("Check Latest Activity")
                .with_description("Display the latest commit time for each student")
                .with_icon("⏰"))
            .item(MenuItem::new("Back")
                .with_description("Return to class management menu")
                .with_icon("↩️"))
            .build();

        Self { class, menu }
    }
}

impl Screen for GitHubActivityScreen {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn screen_type(&self) -> ScreenType {
        ScreenType::new(ScreenTypeVariant::GitHubActivity)
            .with_context(ScreenContext::Class(self.class.clone()))
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
            },
            KeyCode::Down | KeyCode::Char('j') => {
                self.menu.select_next();
                Ok(None)
            },
            KeyCode::Enter | KeyCode::Char(' ') => {
                if let Some(selected_item) = self.menu.selected_item() {
                    match selected_item.title.as_str() {
                        "Week View" => Ok(Some(AppEvent::ShowWeekView)),
                        "Check Latest Activity" => Ok(Some(AppEvent::ShowLatestActivity)),
                        "Back" => Ok(Some(AppEvent::GoBack)),
                        _ => Ok(None),
                    }
                } else {
                    Ok(None)
                }
            },
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
        self.menu.update(delta_time, &AnimationState::new());
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
        let popup_area = crate::ui::layout::center_rect(60, 80, area);
        
        // Create a block for the content
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!("GitHub Activity for Class: {}", self.class.name))
            .title_alignment(Alignment::Center)
            .style(Style::default().bg(theme.background).fg(theme.text));
            
        frame.render_widget(block, popup_area);
        
        // Create inner area for content
        let inner_area = popup_area.inner(&crate::ui::layout::margin(1, 1));
        
        // Render the menu
        let menu_area = inner_area;
        
        // Render menu title
        let title = Paragraph::new("Options")
            .alignment(Alignment::Center)
            .style(Style::default().add_modifier(Modifier::BOLD));
            
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(3),    // Menu
                Constraint::Length(1), // Help text
            ])
            .split(menu_area);
            
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