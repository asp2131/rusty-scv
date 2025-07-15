use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Row, Table, TableState},
    Frame,
};
use std::{collections::HashMap, future::Future, pin::Pin, time::Duration};
use chrono::{DateTime, Utc};

use crate::app::{AppEvent, AppState};
use crate::data::github::GitHubClient;
use crate::data::models::Student;
use crate::ui::{
    animations::AnimationState,
    screens::{Screen, ScreenType, ScreenTypeVariant},
    themes::Theme,
};

pub struct LatestActivityScreen {
    students: Vec<Student>,
    latest_activity_data: HashMap<String, Option<DateTime<Utc>>>,
    table_state: TableState,
    is_loading: bool,
    error_message: Option<String>,
}

impl LatestActivityScreen {
    pub fn new(students: Vec<Student>) -> Self {
        let mut table_state = TableState::default();
        if !students.is_empty() {
            table_state.select(Some(0));
        }

        Self {
            students,
            latest_activity_data: HashMap::new(),
            table_state,
            is_loading: false,
            error_message: None,
        }
    }

    pub fn render(&mut self, f: &mut Frame<ratatui::backend::CrosstermBackend<std::io::Stdout>>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        // Title
        let title = Paragraph::new("GitHub Latest Activity")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan));
        f.render_widget(title, chunks[0]);

        // Main content area
        if self.is_loading {
            self.render_loading(f, chunks[1]);
        } else if let Some(error) = &self.error_message {
            self.render_error(f, chunks[1], error);
        } else {
            self.render_table(f, chunks[1]);
        }

        // Instructions
        let instructions = Paragraph::new("↑/↓: Navigate  r: Refresh timestamps  q: Back")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Gray));
        f.render_widget(instructions, chunks[2]);
    }

    fn render_loading(&self, f: &mut Frame<ratatui::backend::CrosstermBackend<std::io::Stdout>>, area: Rect) {
        let loading_text = Paragraph::new("Loading latest activity data...")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(loading_text, area);
    }

    fn render_error(&self, f: &mut Frame<ratatui::backend::CrosstermBackend<std::io::Stdout>>, area: Rect, error: &str) {
        let error_text = Paragraph::new(format!("Error: {}", error))
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Red));
        f.render_widget(error_text, area);
    }

    fn render_table(&mut self, f: &mut Frame<ratatui::backend::CrosstermBackend<std::io::Stdout>>, area: Rect) {
        let header = Row::new(vec!["Student", "GitHub Username", "Last Commit"])
            .style(Style::default().fg(Color::Yellow))
            .height(1);

        let rows: Vec<Row> = self.students.iter().map(|student| {
            let github_username = &student.github_username;
            let latest_activity = if let Some(activity) = self.latest_activity_data.get(github_username) {
                if let Some(datetime) = activity {
                    format_time_ago(*datetime)
                } else {
                    "No commits found".to_string()
                }
            } else {
                "Loading...".to_string()
            };

            Row::new(vec![
                student.username.clone(),
                github_username.clone(),
                latest_activity,
            ])
        }).collect();

        let table = Table::new(rows)
        .widths(&[
            Constraint::Length(20),
            Constraint::Length(25),
            Constraint::Min(25),
        ])
        .header(header)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(Style::default().fg(Color::Black).bg(Color::White));

        f.render_stateful_widget(table, area, &mut self.table_state);
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<AppEvent>> {
        match key.code {
            KeyCode::Up => {
                if let Some(selected) = self.table_state.selected() {
                    if selected > 0 {
                        self.table_state.select(Some(selected - 1));
                    }
                }
                Ok(None)
            }
            KeyCode::Down => {
                if let Some(selected) = self.table_state.selected() {
                    if selected < self.students.len().saturating_sub(1) {
                        self.table_state.select(Some(selected + 1));
                    }
                } else if !self.students.is_empty() {
                    self.table_state.select(Some(0));
                }
                Ok(None)
            }
            KeyCode::Char('r') => {
                Ok(Some(AppEvent::RefreshLatestActivity))
            }
            KeyCode::Char('q') => {
                Ok(Some(AppEvent::GoBack))
            }
            _ => Ok(None),
        }
    }

    pub async fn load_activity_data(&mut self, github_client: &GitHubClient) -> Result<()> {
        self.is_loading = true;
        self.error_message = None;

        let mut activity_data = HashMap::new();

        for student in &self.students {
            let github_username = &student.github_username;
            match github_client.get_latest_activity(github_username).await {
                Ok(latest_activity) => {
                    activity_data.insert(github_username.clone(), latest_activity);
                }
                Err(e) => {
                    eprintln!("Error fetching latest activity for {}: {}", github_username, e);
                    activity_data.insert(github_username.clone(), None);
                }
            }
        }

        self.latest_activity_data = activity_data;
        self.is_loading = false;
        Ok(())
    }

    pub fn set_error(&mut self, error: String) {
        self.error_message = Some(error);
        self.is_loading = false;
    }
}

impl Screen for LatestActivityScreen {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn screen_type(&self) -> ScreenType {
        ScreenType::new(ScreenTypeVariant::LatestActivity)
    }

    fn handle_key_event<'a>(
        &'a mut self,
        key: KeyEvent,
        _state: &'a AppState,
    ) -> Pin<Box<dyn Future<Output = Result<Option<AppEvent>>> + Send + 'a>> {
        let result = self.handle_key_event(key);
        Box::pin(async move { result })
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
        _theme: &Theme,
    ) {
        self.render(frame, area);
    }
}

fn format_time_ago(datetime: DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(datetime);
    
    let seconds = duration.num_seconds();
    let minutes = duration.num_minutes();
    let hours = duration.num_hours();
    let days = duration.num_days();
    
    if seconds < 60 {
        if seconds <= 1 {
            "just now".to_string()
        } else {
            format!("{} seconds ago", seconds)
        }
    } else if minutes < 60 {
        if minutes == 1 {
            "1 minute ago".to_string()
        } else {
            format!("{} minutes ago", minutes)
        }
    } else if hours < 24 {
        if hours == 1 {
            "1 hour ago".to_string()
        } else {
            format!("{} hours ago", hours)
        }
    } else if days < 7 {
        if days == 1 {
            "1 day ago".to_string()
        } else {
            format!("{} days ago", days)
        }
    } else if days < 30 {
        let weeks = days / 7;
        if weeks == 1 {
            "1 week ago".to_string()
        } else {
            format!("{} weeks ago", weeks)
        }
    } else if days < 365 {
        let months = days / 30;
        if months == 1 {
            "1 month ago".to_string()
        } else {
            format!("{} months ago", months)
        }
    } else {
        let years = days / 365;
        if years == 1 {
            "1 year ago".to_string()
        } else {
            format!("{} years ago", years)
        }
    }
}