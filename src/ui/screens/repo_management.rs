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

use crate::{
    app::{AppEvent, AppState},
    data::{Class, Student},
    git::GitManager,
    ui::{
        animations::AnimationState,
        components::menu::{AnimatedMenu, MenuBuilder, MenuItem},
        screens::{Screen, ScreenType, ScreenTypeVariant},
        themes::Theme,
    },
};

pub struct RepoManagementScreen {
    class: Class,
    students: Vec<Student>,
    menu: AnimatedMenu,
    selected_index: usize,
    show_actions: bool,
    show_main_menu: bool,
}

impl RepoManagementScreen {
    pub fn new(class: Class, students: Vec<Student>) -> Self {
        let menu = MenuBuilder::new()
            .title(format!("Repository Management - {}", class.name))
            .item(MenuItem::new("Clone All Repositories").with_description("Clone all student repositories").with_icon("📥"))
            .item(MenuItem::new("Individual Student Actions").with_description("Select individual student for actions").with_icon("👤"))
            .item(MenuItem::new("Back").with_description("Return to class management").with_icon("↩️"))
            .build();

        Self {
            class,
            students,
            menu,
            selected_index: 0,
            show_actions: false,
            show_main_menu: true,
        }
    }

    fn get_selected_student(&self) -> Option<&Student> {
        self.students.get(self.selected_index)
    }

    fn update_menu_for_student_username(&mut self, github_username: &str) {
        self.menu = MenuBuilder::new()
            .title(format!("Repository Actions for {}", github_username))
            .item(MenuItem::new("Clone Repo").with_description("Clone GitHub Pages repo").with_icon("📥"))
            .item(MenuItem::new("Pull Repo").with_description("Pull latest changes from remote").with_icon("🔄"))
            .item(MenuItem::new("Clean Repo").with_description("Reset local changes to match remote").with_icon("🧹"))
            .item(MenuItem::new("Open in Terminal").with_description("Open terminal at repo location").with_icon("🖥️"))
            .item(MenuItem::new("Back").with_description("Return to student selection").with_icon("↩️"))
            .build();
    }
}

impl Screen for RepoManagementScreen {
    fn screen_type(&self) -> ScreenType {
        ScreenType::new(ScreenTypeVariant::RepositoryManagement)
            .with_context(crate::ui::screens::ScreenContext::Class(self.class.clone()))
    }

    fn handle_key_event<'a>(
        &'a mut self,
        key: KeyEvent,
        state: &'a AppState,
    ) -> Pin<Box<dyn Future<Output = Result<Option<AppEvent>>> + Send + 'a>> {
        let result = if self.show_main_menu {
            // Handle main menu
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    self.menu.select_previous();
                    Ok(None)
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.menu.select_next();
                    Ok(None)
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    if let Some(item) = self.menu.selected_item() {
                        match item.title.as_str() {
                            "Clone All Repositories" => Ok(Some(AppEvent::CloneAllRepos)),
                            "Individual Student Actions" => {
                                self.show_main_menu = false;
                                Ok(None)
                            }
                            "Back" => Ok(Some(AppEvent::GoBack)),
                            _ => Ok(None),
                        }
                    } else {
                        Ok(None)
                    }
                }
                KeyCode::Esc => Ok(Some(AppEvent::GoBack)),
                _ => Ok(None),
            }
        } else if self.show_actions {
            // Handle actions menu
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    self.menu.select_previous();
                    Ok(None)
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.menu.select_next();
                    Ok(None)
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    if let Some(selected_student) = self.get_selected_student() {
                        if let Some(item) = self.menu.selected_item() {
                            match item.title.as_str() {
                                "Clone Repo" => Ok(Some(AppEvent::CloneRepo(selected_student.github_username.clone()))),
                                "Pull Repo" => Ok(Some(AppEvent::PullRepo(selected_student.github_username.clone()))),
                                "Clean Repo" => Ok(Some(AppEvent::CleanRepo(selected_student.github_username.clone()))),
                                "Open in Terminal" => Ok(Some(AppEvent::OpenInTerminal(selected_student.github_username.clone()))),
                                "Back" => {
                                    self.show_actions = false;
                                    Ok(None)
                                }
                                _ => Ok(None),
                            }
                        } else {
                            Ok(None)
                        }
                    } else {
                        Ok(None)
                    }
                }
                KeyCode::Esc => {
                    self.show_actions = false;
                    Ok(None)
                }
                _ => Ok(None),
            }
        } else {
            // Handle student selection
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    }
                    Ok(None)
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.selected_index + 1 < self.students.len() {
                        self.selected_index += 1;
                    }
                    Ok(None)
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    if let Some(selected_student) = self.get_selected_student() {
                        let github_username = selected_student.github_username.clone();
                        // Switch to actions menu
                        self.show_actions = true;
                        self.update_menu_for_student_username(&github_username);
                        Ok(None)
                    } else {
                        Ok(None)
                    }
                }
                KeyCode::Esc => {
                    self.show_main_menu = true;
                    Ok(None)
                }
                _ => Ok(None),
            }
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
        state: &AppState,
        animation_state: &AnimationState,
        theme: &Theme,
    ) {
        if self.show_main_menu || self.show_actions {
            // Render main menu or actions menu
            frame.render_widget(&mut self.menu, area);
        } else {
            // Render student selection
            let block = Block::default()
                .borders(Borders::ALL)
                .title(format!("Select Student for Repository Actions - {}", self.class.name));
            let inner_area = block.inner(area);
            frame.render_widget(block, area);
            
            // Check if we have students
            if self.students.is_empty() {
                let no_students_text = Paragraph::new("No students found in this class.\n\nPress ESC to go back.")
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(theme.text_secondary));
                frame.render_widget(no_students_text, inner_area);
                return;
            }
            
            let student_list: Vec<Line> = self.students.iter().enumerate().map(|(i, student)| {
                let style = if i == self.selected_index {
                    Style::default().fg(theme.highlight).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text)
                };
                
                // Show repository status
                let repo_status = if state.git_manager.repo_exists(&student.github_username, &self.class.name) {
                    "✓ Cloned"
                } else {
                    "✗ Not cloned"
                };
                
                let prefix = if i == self.selected_index { "▶ " } else { "  " };
                
                Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(
                        format!("{} ({})", student.github_username, student.username),
                        style
                    ),
                    Span::styled(
                        format!(" [{}]", repo_status),
                        if repo_status.starts_with("✓") {
                            Style::default().fg(theme.success)
                        } else {
                            Style::default().fg(theme.text_secondary)
                        }
                    ),
                ])
            }).collect();
            
            let student_paragraph = Paragraph::new(student_list)
                .alignment(Alignment::Left);
            frame.render_widget(student_paragraph, inner_area);
            
            // Show help text
            let help_area = Rect {
                x: inner_area.x,
                y: inner_area.y + inner_area.height.saturating_sub(2),
                width: inner_area.width,
                height: 2,
            };
            
            let help_text = vec![
                Line::from(vec![
                    Span::styled("↑/↓ or j/k", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                    Span::styled(" Navigate  ", Style::default().fg(theme.text_secondary)),
                    Span::styled("Enter", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                    Span::styled(" Select  ", Style::default().fg(theme.text_secondary)),
                    Span::styled("ESC", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                    Span::styled(" Back", Style::default().fg(theme.text_secondary)),
                ])
            ];
            
            let help_paragraph = Paragraph::new(help_text)
                .alignment(Alignment::Center);
            frame.render_widget(help_paragraph, help_area);
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
