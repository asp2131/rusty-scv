use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame, 
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    backend::Backend,
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
        components::{
            loading::LoadingWidget,
            menu::{AnimatedMenu, MenuBuilder, MenuItem, MenuPresets},
        },
        screens::{Screen, ScreenType, ScreenTypeVariant},
        themes::Theme,
    },
};

pub struct ClassSelectionScreen {
    classes: Vec<Class>,
    menu: AnimatedMenu,
    loading: bool,
    needs_refresh: bool,
    error: Option<String>,
}

impl ClassSelectionScreen {
    pub fn new() -> Self {
        Self {
            classes: Vec::new(),
            menu: MenuBuilder::new()
                .title("📚 Select a Class")
                .simple_item("Loading...")
                .build(),
            loading: true,
            needs_refresh: true,
            error: None,
        }
    }
    
    pub fn needs_refresh(&self) -> bool {
        self.needs_refresh
    }
    
    pub fn set_classes(&mut self, classes: Vec<Class>) {
        self.classes = classes;
        self.menu = Self::build_class_menu(&self.classes);
        self.menu.trigger_entrance();
        self.loading = false;
        self.needs_refresh = false;
        self.error = None;
    }
    
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
        self.loading = false;
        self.needs_refresh = false;
    }
    
    async fn refresh_classes(&mut self, state: &AppState) -> Result<()> {
        self.loading = true;
        match state.database.get_classes().await {
            Ok(classes) => {
                self.classes = classes;
                self.menu = Self::build_class_menu(&self.classes);
                self.menu.trigger_entrance();
                self.error = None;
            }
            Err(e) => {
                self.error = Some(format!("Failed to load classes: {}", e));
            }
        }
        self.loading = false;
        Ok(())
    }
    
    fn build_class_menu(classes: &[Class]) -> AnimatedMenu {
        let mut builder = MenuBuilder::new()
            .title("📚 Select a Class");
            
        if classes.is_empty() {
            // Show empty state with option to create a class
            builder = builder
                .item(MenuItem::new("No classes found")
                    .with_description("Press 'n' to create your first class")
                    .with_icon("ℹ️"));
        } else {
            // Add each class to the menu
            for class in classes {
                builder = builder.item(MenuItem::new(&class.name)
                    .with_description(&format!("Manage class: {}", class.name))
                    .with_icon("📖"));
            }
        }
        
        builder
            .item(MenuItem::new("Create New Class")
                .with_description("Add a new class")
                .with_icon("➕")
                .with_hotkey('c'))
            .item(MenuItem::new("Back")
                .with_description("Return to main menu")
                .with_icon("↩️")
                .with_hotkey('b'))
            .build()
    }
}

impl Screen for ClassSelectionScreen {
    fn screen_type(&self) -> ScreenType {
        ScreenType::new(ScreenTypeVariant::ClassSelection)
    }

    fn handle_key_event<'a>(
        &'a mut self,
        key: KeyEvent,
        _state: &'a AppState,
    ) -> Pin<Box<dyn Future<Output = Result<Option<AppEvent>>> + Send + 'a>> {
        // Handle quit
        if key.code == KeyCode::Char('q') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return Box::pin(async { Ok(Some(AppEvent::Quit)) });
        }
        
        // Handle navigation
        match key.code {
            // Navigation keys
            KeyCode::Char('j') | KeyCode::Down => {
                self.menu.select_next();
            },
            KeyCode::Char('k') | KeyCode::Up => {
                self.menu.select_previous();
            },
            // Enter key for selection
            KeyCode::Enter | KeyCode::Char(' ') => {
                if self.classes.is_empty() {
                    // If no classes, allow creating a new one
                    return Box::pin(async { 
                        Ok(Some(AppEvent::NavigateToScreen(ScreenType::new(ScreenTypeVariant::CreateClass)))) 
                    });
                } else if let Some(selected_item) = self.menu.selected_item() {
                    // Find the selected class
                    if let Some(class) = self.classes.iter().find(|c| c.name == selected_item.title) {
                        return Box::pin(async { 
                            Ok(Some(AppEvent::SelectClass(class.clone()))) 
                        });
                    }
                }
            },
            // Create new class
            KeyCode::Char('n') => {
                return Box::pin(async { 
                    Ok(Some(AppEvent::NavigateToScreen(ScreenType::new(ScreenTypeVariant::CreateClass)))) 
                });
            },
            // Refresh class list
            KeyCode::Char('r') => {
                return Box::pin(async { 
                    Ok(Some(AppEvent::RefreshData)) 
                });
            },
            // Go back to main menu
            KeyCode::Esc => {
                return Box::pin(async { 
                    Ok(Some(AppEvent::NavigateToScreen(ScreenType::new(ScreenTypeVariant::MainMenu))))
                });
            },
            _ => {}
        }
        
        Box::pin(async { Ok(None) })
    }

    fn update<'a>(
        &'a mut self,
        delta_time: Duration,
        _state: &'a mut AppState,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        // Update menu animation
        let animation_state = AnimationState::default();
        self.menu.update(delta_time, &animation_state);
        
        // Note: We'll handle refresh through app events instead of direct database calls
        // since the Screen trait requires Send futures but AppState is not Sync
        
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
        let popup_area = crate::ui::layout::center_rect(60, 80, area);
        
        // Create a block for the content
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Select a Class")
            .title_alignment(Alignment::Center)
            .style(Style::default().bg(theme.background).fg(theme.text));
            
        frame.render_widget(block, popup_area);
        
        // Create inner area for content
        let inner_area = popup_area.inner(&crate::ui::layout::margin(1, 1));
        
        if self.loading {
            // Render loading state
            let loading_widget = crate::ui::components::loading::LoadingPresets::initializing(theme);
            frame.render_widget(loading_widget, inner_area);
        } else if let Some(error) = &self.error {
            // Render error message
            let error_text = Line::from(Span::styled(
                error,
                Style::default().fg(theme.error),
            ));
            frame.render_widget(
                Paragraph::new(error_text)
                    .alignment(Alignment::Center),
                inner_area,
            );
        } else {
            // Render the menu
            let menu_area = inner_area;
            
            // Render menu title
            let title = Paragraph::new("Available Classes")
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
                Span::styled("n", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(": New Class  "),
                Span::styled("r", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(": Refresh  "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(": Quit"),
            ]);
            
            frame.render_widget(
                Paragraph::new(help_text)
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(theme.text_secondary)),
                chunks[2],
            );
        }
    }
}