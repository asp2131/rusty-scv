use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::{future::Future, pin::Pin, time::Duration};

use crate::{
    app::{AppEvent, AppState},
    data::Class,
    ui::{
        animations::AnimationState,
        screens::{Screen, ScreenType, ScreenTypeVariant},
        themes::Theme,
    },
};

pub struct ClassManagementScreen {
    class: Class,
    selected: usize,
    menu_items: Vec<MenuOption>,
}

#[derive(Clone)]
struct MenuOption {
    title: String,
    description: String,
    icon: String,
    hotkey: char,
}

impl ClassManagementScreen {
    pub fn new(class: Class) -> Self {
        let menu_items = vec![
            MenuOption {
                title: "Manage Students".to_string(),
                description: "Add or remove students".to_string(),
                icon: "👥".to_string(),
                hotkey: 's',
            },
            MenuOption {
                title: "Manage Repositories".to_string(),
                description: "Clone, pull, or clean repositories".to_string(),
                icon: "📁".to_string(),
                hotkey: 'r',
            },
            MenuOption {
                title: "View GitHub Activity".to_string(),
                description: "Check student GitHub activity".to_string(),
                icon: "📊".to_string(),
                hotkey: 'a',
            },
            MenuOption {
                title: "Delete Class".to_string(),
                description: "Delete this class and its data".to_string(),
                icon: "🗑️".to_string(),
                hotkey: 'd',
            },
            MenuOption {
                title: "Back".to_string(),
                description: "Return to main menu".to_string(),
                icon: "↩️".to_string(),
                hotkey: 'b',
            },
        ];
        
        Self { 
            class, 
            selected: 0,
            menu_items,
        }
    }

    fn select_next(&mut self) {
        if !self.menu_items.is_empty() {
            self.selected = (self.selected + 1) % self.menu_items.len();
        }
    }

    fn select_previous(&mut self) {
        if !self.menu_items.is_empty() {
            self.selected = if self.selected == 0 {
                self.menu_items.len() - 1
            } else {
                self.selected - 1
            };
        }
    }

    fn get_selected_item(&self) -> Option<&MenuOption> {
        self.menu_items.get(self.selected)
    }
}

impl Screen for ClassManagementScreen {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

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
                self.select_previous();
                Ok(None)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.select_next();
                Ok(None)
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                if let Some(selected) = self.get_selected_item() {
                    match selected.title.as_str() {
                        "Manage Students" => Ok(Some(AppEvent::NavigateToScreen(ScreenType::new(ScreenTypeVariant::StudentManagement).with_context(crate::ui::screens::ScreenContext::Class(self.class.clone()))))),
                        "Manage Repositories" => Ok(Some(AppEvent::ShowError("Repository management not implemented yet".to_string()))),
                        "View GitHub Activity" => Ok(Some(AppEvent::NavigateToScreen(ScreenType::new(ScreenTypeVariant::GitHubActivity).with_context(crate::ui::screens::ScreenContext::Class(self.class.clone()))))),
                        "Delete Class" => Ok(Some(AppEvent::ShowError("Delete class not implemented yet".to_string()))),
                        "Back" => Ok(Some(AppEvent::GoBack)),
                        _ => Ok(None),
                    }
                } else {
                    Ok(None)
                }
            }
            KeyCode::Char('s') => Ok(Some(AppEvent::NavigateToScreen(ScreenType::new(ScreenTypeVariant::StudentManagement).with_context(crate::ui::screens::ScreenContext::Class(self.class.clone()))))),
            KeyCode::Char('r') => Ok(Some(AppEvent::ShowError("Repository management not implemented yet".to_string()))),
            KeyCode::Char('a') => Ok(Some(AppEvent::NavigateToScreen(ScreenType::new(ScreenTypeVariant::GitHubActivity).with_context(crate::ui::screens::ScreenContext::Class(self.class.clone()))))),
            KeyCode::Char('d') => Ok(Some(AppEvent::ShowError("Delete class not implemented yet".to_string()))),
            KeyCode::Char('b') | KeyCode::Esc => Ok(Some(AppEvent::GoBack)),
            _ => Ok(None),
        };

        Box::pin(async { result })
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
        let popup_area = crate::ui::layout::center_rect(80, 70, area);
        
        // Create outer block
        let outer_block = Block::default()
            .borders(Borders::ALL)
            .title(format!("📚 Managing Class: {}", self.class.name))
            .title_alignment(Alignment::Center)
            .border_style(theme.border_focused_style())
            .style(Style::default().bg(theme.background));
            
        // Get inner area before rendering the block
        let inner_area = outer_block.inner(popup_area);
        frame.render_widget(outer_block, popup_area);
        
        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),  // Class info
                Constraint::Min(5),     // Menu
                Constraint::Length(3),  // Help
            ])
            .split(inner_area);
        
        // Render class info
        let class_info = Paragraph::new(format!("Class ID: {} • Created: {}", self.class.id, self.class.created_at.format("%Y-%m-%d")))
            .alignment(Alignment::Center)
            .style(Style::default().fg(theme.text_secondary));
        frame.render_widget(class_info, chunks[0]);
        
        // Create menu items for the list
        let list_items: Vec<ListItem> = self.menu_items.iter().enumerate().map(|(i, item)| {
            let is_selected = i == self.selected;
            
            let style = if is_selected {
                Style::default().bg(theme.highlight).fg(theme.text).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };
            
            let prefix = if is_selected { "▶ " } else { "  " };
            let content = format!("{}{} {} - {}", prefix, item.icon, item.title, item.description);
            
            ListItem::new(content).style(style)
        }).collect();
        
        // Create and render the list
        let list = List::new(list_items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Options")
                .border_style(theme.border_style()))
            .highlight_style(Style::default().bg(theme.selection))
            .highlight_symbol(">> ");
            
        let mut list_state = ListState::default();
        list_state.select(Some(self.selected));
        
        frame.render_stateful_widget(list, chunks[1], &mut list_state);
        
        // Render help
        let help_lines = vec![
            Line::from(vec![
                Span::styled("↑/↓ or k/j", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                Span::styled(": Navigate  ", Style::default().fg(theme.text_secondary)),
                Span::styled("Enter", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                Span::styled(": Select  ", Style::default().fg(theme.text_secondary)),
                Span::styled("Esc", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                Span::styled(": Back", Style::default().fg(theme.text_secondary)),
            ]),
            Line::from(vec![
                Span::styled("Hotkeys: ", Style::default().fg(theme.text_secondary)),
                Span::styled("s", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::styled(":Students  ", Style::default().fg(theme.text_secondary)),
                Span::styled("r", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::styled(":Repos  ", Style::default().fg(theme.text_secondary)),
                Span::styled("a", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::styled(":Activity  ", Style::default().fg(theme.text_secondary)),
                Span::styled("d", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::styled(":Delete", Style::default().fg(theme.text_secondary)),
            ]),
        ];
        
        let help = Paragraph::new(help_lines)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::TOP))
            .style(Style::default().fg(theme.text_secondary));
        frame.render_widget(help, chunks[2]);
    }
}