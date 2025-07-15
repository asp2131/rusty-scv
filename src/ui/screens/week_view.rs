use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
};
use std::{future::Future, pin::Pin, time::Duration};

use crate::{
    app::{AppEvent, AppState},
    data::{Class, Student, github::{WeekActivity, GitHubClient, format_weekday, get_current_weekdays}},
    ui::{
        animations::AnimationState,
        screens::{Screen, ScreenType, ScreenTypeVariant, ScreenContext},
        themes::Theme,
    },
};

pub struct WeekViewScreen {
    class: Class,
    students: Vec<Student>,
    activities: Vec<WeekActivity>,
    loading: bool,
    error: Option<String>,
    table_state: TableState,
}

impl WeekViewScreen {
    pub fn new(class: Class, students: Vec<Student>) -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0));
        
        Self {
            class,
            students,
            activities: Vec::new(),
            loading: false,
            error: None,
            table_state,
        }
    }

    pub async fn load_activity_data(&mut self, github_token: Option<String>) {
        self.loading = true;
        self.error = None;
        
        let github_client = GitHubClient::new(github_token);
        let mut activities = Vec::new();
        
        for student in &self.students {
            match github_client.get_week_activity(&student.github_username).await {
                Ok(activity) => {
                    activities.push(activity);
                }
                Err(e) => {
                    activities.push(WeekActivity {
                        student_username: student.username.clone(),
                        student_github_username: student.github_username.clone(),
                        daily_commits: std::collections::HashMap::new(),
                        total_commits: 0,
                        latest_commit: None,
                        error: Some(e.to_string()),
                    });
                }
            }
        }
        
        self.activities = activities;
        self.loading = false;
    }

    fn create_table_rows(&self) -> Vec<Row> {
        Self::create_table_rows_static(&self.activities)
    }

    fn create_table_rows_static(activities: &[WeekActivity]) -> Vec<Row> {
        let weekdays = get_current_weekdays();
        let mut rows = Vec::new();
        
        for activity in activities {
            let mut cells = vec![
                Cell::from(activity.student_username.clone()),
            ];
            
            // Add cells for each weekday
            for weekday in &weekdays {
                let symbol = if let Some(_error) = &activity.error {
                    "❌"
                } else if *activity.daily_commits.get(weekday).unwrap_or(&false) {
                    "✅"
                } else {
                    "❌"
                };
                cells.push(Cell::from(symbol));
            }
            
            // Add total commits cell
            let total_text = if activity.error.is_some() {
                "Error".to_string()
            } else {
                activity.total_commits.to_string()
            };
            cells.push(Cell::from(total_text));
            
            rows.push(Row::new(cells));
        }
        
        rows
    }

    fn create_table_header() -> Row<'static> {
        let weekdays = get_current_weekdays();
        let mut header_cells = vec![Cell::from("Student").style(Style::default().add_modifier(Modifier::BOLD))];
        
        for weekday in &weekdays {
            header_cells.push(Cell::from(format_weekday(*weekday)).style(Style::default().add_modifier(Modifier::BOLD)));
        }
        
        header_cells.push(Cell::from("Total").style(Style::default().add_modifier(Modifier::BOLD)));
        
        Row::new(header_cells)
    }
}

impl Screen for WeekViewScreen {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn screen_type(&self) -> ScreenType {
        ScreenType::new(ScreenTypeVariant::WeekView)
            .with_context(ScreenContext::Class(self.class.clone()))
    }

    fn handle_key_event<'a>(
        &'a mut self,
        key: KeyEvent,
        _state: &'a AppState,
    ) -> Pin<Box<dyn Future<Output = Result<Option<AppEvent>>> + Send + 'a>> {
        let result = match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                let selected = self.table_state.selected().unwrap_or(0);
                if selected > 0 {
                    self.table_state.select(Some(selected - 1));
                }
                Ok(None)
            },
            KeyCode::Down | KeyCode::Char('j') => {
                let selected = self.table_state.selected().unwrap_or(0);
                if selected + 1 < self.activities.len() {
                    self.table_state.select(Some(selected + 1));
                }
                Ok(None)
            },
            KeyCode::Char('r') => {
                // Refresh data
                Ok(Some(AppEvent::RefreshData))
            },
            KeyCode::Esc => {
                Ok(Some(AppEvent::GoBack))
            },
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
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!("Week View - {} (Past 5 Weekdays)", self.class.name))
            .title_alignment(Alignment::Center)
            .style(Style::default().bg(theme.background).fg(theme.text));

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        if self.loading {
            let loading_text = Paragraph::new("Loading GitHub activity data...")
                .alignment(Alignment::Center)
                .style(Style::default().fg(theme.text_secondary));
            frame.render_widget(loading_text, inner_area);
            return;
        }

        if let Some(error) = &self.error {
            let error_text = Paragraph::new(format!("Error: {}", error))
                .alignment(Alignment::Center)
                .style(Style::default().fg(theme.error));
            frame.render_widget(error_text, inner_area);
            return;
        }

        if self.activities.is_empty() {
            let empty_text = Paragraph::new("No students found in this class.")
                .alignment(Alignment::Center)
                .style(Style::default().fg(theme.text_secondary));
            frame.render_widget(empty_text, inner_area);
            return;
        }

        // Create layout for table and help text
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),     // Table area
                Constraint::Length(3),  // Help text
            ])
            .split(inner_area);

        // Create table rendering separately to avoid borrow checker issues
        let activities = &self.activities;
        let table = {
            let header = Self::create_table_header();
            let rows = Self::create_table_rows_static(activities);
            
            Table::new(rows)
                .header(header)
                .block(Block::default().borders(Borders::NONE))
                .style(Style::default().fg(theme.text))
                .highlight_style(Style::default().bg(theme.highlight).fg(theme.background))
                .highlight_symbol("▶ ")
                .widths(&[
                    Constraint::Length(20), // Student name
                    Constraint::Length(5),  // Mon
                    Constraint::Length(5),  // Tue
                    Constraint::Length(5),  // Wed
                    Constraint::Length(5),  // Thu
                    Constraint::Length(5),  // Fri
                    Constraint::Length(8),  // Total
                ])
        };

        // Render the table using the state
        frame.render_stateful_widget(table, chunks[0], &mut self.table_state);

        // Help text
        let help_text = vec![
            Line::from(vec![
                Span::styled("↑/↓", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                Span::styled(" Navigate  ", Style::default().fg(theme.text_secondary)),
                Span::styled("r", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                Span::styled(" Refresh  ", Style::default().fg(theme.text_secondary)),
                Span::styled("ESC", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                Span::styled(" Back", Style::default().fg(theme.text_secondary)),
            ]),
            Line::from(vec![
                Span::styled("✅", Style::default().fg(theme.success)),
                Span::styled(" Committed  ", Style::default().fg(theme.text_secondary)),
                Span::styled("❌", Style::default().fg(theme.error)),
                Span::styled(" No commits", Style::default().fg(theme.text_secondary)),
            ]),
        ];

        let help_paragraph = Paragraph::new(help_text)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::TOP));

        frame.render_widget(help_paragraph, chunks[1]);
    }
}