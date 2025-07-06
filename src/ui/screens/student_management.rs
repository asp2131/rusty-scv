use anyhow::Result;
use crossterm::event::{KeyEvent, KeyCode};
use ratatui::{
    Frame, backend::Backend, 
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use std::{future::Future, pin::Pin, time::Duration};

use crate::{
    app::{AppEvent, AppState},
    data::Class,
    ui::{animations::AnimationState, themes::Theme},
};

use super::{Screen, ScreenType, ScreenTypeVariant, ScreenContext};

struct MenuOption {
    title: String,
    description: String,
    action: String,
}

impl MenuOption {
    fn new(title: &str, description: &str, action: &str) -> Self {
        Self {
            title: title.to_string(),
            description: description.to_string(),
            action: action.to_string(),
        }
    }
}

pub struct StudentManagementScreen {
    class: Class,
    menu_options: Vec<MenuOption>,
    selected_index: usize,
}

impl StudentManagementScreen {
    pub fn new(class: Class) -> Self {
        let menu_options = vec![
            MenuOption::new("Add Student(s)", "Add new students to this class", "add"),
            MenuOption::new("Delete Student", "Remove a student from this class", "delete"),
            MenuOption::new("Back", "Return to class management menu", "back"),
        ];

        Self {
            class,
            menu_options,
            selected_index: 0,
        }
    }
}

impl Screen for StudentManagementScreen {
    fn screen_type(&self) -> ScreenType {
        ScreenType::new(ScreenTypeVariant::StudentManagement)
            .with_context(ScreenContext::Class(self.class.clone()))
    }

    fn handle_key_event(&mut self, key: KeyEvent, state: &AppState) -> Pin<Box<dyn Future<Output = Result<Option<AppEvent>>> + Send + '_>> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Box::pin(async move { Ok(Some(AppEvent::GoBack)) }),
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_index = (self.selected_index + self.menu_options.len() - 1) % self.menu_options.len();
                Box::pin(async move { Ok(None) })
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.selected_index = (self.selected_index + 1) % self.menu_options.len();
                Box::pin(async move { Ok(None) })
            }
            KeyCode::Enter => {
                match self.menu_options[self.selected_index].action.as_str() {
                    "add" => Box::pin(async move {
                        Ok(Some(AppEvent::NavigateToScreen(
                            ScreenType::new(ScreenTypeVariant::AddStudents)
                                .with_context(ScreenContext::Class(self.class.clone()))
                        )))
                    }),
                    "delete" => Box::pin(async move {
                        Ok(Some(AppEvent::NavigateToScreen(
                            ScreenType::new(ScreenTypeVariant::DeleteStudent)
                                .with_context(ScreenContext::Class(self.class.clone()))
                        )))
                    }),
                    "back" => Box::pin(async move { Ok(Some(AppEvent::GoBack)) }),
                    _ => Box::pin(async move { Ok(None) }),
                }
            }
            _ => Box::pin(async move { Ok(None) }),
        }
    }

    fn update<'a>(&'a mut self, _delta_time: Duration, _state: &'a mut AppState) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move { Ok(()) })
    }

    fn render(
        &mut self, 
        frame: &mut ratatui::Frame<ratatui::backend::CrosstermBackend<std::io::Stdout>>, 
        area: Rect, 
        state: &AppState, 
        _animation_state: &AnimationState, 
        theme: &Theme
    ) {
        // Clear the area first
        frame.render_widget(Clear, area);

        // Create a centered block for the menu
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!("Manage Students: {}", self.class.name))
            .border_style(Style::default().fg(theme.primary));

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        // Render menu options
        let menu_items: Vec<Line> = self.menu_options.iter().enumerate().map(|(i, option)| {
            let is_selected = i == self.selected_index;
            let style = if is_selected {
                Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };

            Line::from(vec![
                Span::styled(
                    if is_selected { "→ " } else { "  " },
                    style,
                ),
                Span::styled(option.title.clone(), style),
                Span::styled(format!("\n    {}", option.description), Style::default().fg(theme.text_secondary)),
            ])
        }).collect();

        let menu = Paragraph::new(menu_items)
            .wrap(Wrap { trim: true });

        frame.render_widget(menu, inner_area);

        // Render help text
        let help_text = Line::from(vec![
            Span::styled("↑/k up • ", Style::default().fg(theme.text_secondary)),
            Span::styled("↓/j down • ", Style::default().fg(theme.text_secondary)),
            Span::styled("enter select • ", Style::default().fg(theme.text_secondary)),
            Span::styled("q quit", Style::default().fg(theme.text_secondary)),
        ]);

        let help_paragraph = Paragraph::new(help_text)
            .alignment(Alignment::Center);

        let help_area = Rect {
            x: inner_area.x,
            y: inner_area.y + inner_area.height.saturating_sub(1),
            width: inner_area.width,
            height: 1,
        };

        frame.render_widget(help_paragraph, help_area);
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}