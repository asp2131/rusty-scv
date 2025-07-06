use anyhow::Result;
use crossterm::event::{KeyEvent, KeyCode};
use ratatui::{
    layout::{Alignment, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use tokio::pin;
use std::pin::Pin;

use crate::{
    app::AppEvent,
    data::{Class, Database, Student},
    ui::themes::Theme,
};

pub struct DeleteStudentScreen {
    class: Class,
    students: Vec<Student>,
    selected_index: usize,
}

impl DeleteStudentScreen {
    pub fn new(class: Class, students: Vec<Student>) -> Self {
        Self {
            class,
            students,
            selected_index: 0,
        }
    }
}

impl super::Screen for DeleteStudentScreen {
    fn screen_type(&self) -> super::ScreenType {
        super::ScreenType::new(super::ScreenTypeVariant::DeleteStudent)
            .with_context(super::ScreenContext::Class(self.class.clone()))
    }

    fn update<'a>(&'a mut self, _delta_time: std::time::Duration, _state: &'a mut crate::app::AppState) -> Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move { Ok(()) })
    }

    fn handle_key_event(&mut self, key: KeyEvent, state: &crate::app::AppState) -> Pin<Box<dyn std::future::Future<Output = Result<Option<AppEvent>>> + Send + '_>> {
        match key.code {
            KeyCode::Esc => Box::pin(async move { Ok(Some(AppEvent::GoBack)) }),
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_index = (self.selected_index + self.students.len() - 1) % self.students.len();
                Box::pin(async move { Ok(None) })
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.selected_index = (self.selected_index + 1) % self.students.len();
                Box::pin(async move { Ok(None) })
            }
            KeyCode::Enter => {
                if self.students.is_empty() {
                    return Box::pin(async move { Ok(None) });
                }
                
                let student_id = self.students[self.selected_index].id;
                let db = state.database.clone();
                
                Box::pin(async move {
                    if let Err(e) = db.delete_student(student_id).await {
                        log::error!("Failed to delete student: {}", e);
                    }
                    Ok(Some(AppEvent::GoBack))
                })
            }
            _ => Box::pin(async move { Ok(None) }),
        }
    }

    fn render(
        &mut self, 
        frame: &mut ratatui::Frame<ratatui::backend::CrosstermBackend<std::io::Stdout>>, 
        area: Rect, 
        state: &crate::app::AppState, 
        _animation_state: &crate::ui::animations::AnimationState, 
        theme: &Theme
    ) {
        frame.render_widget(Clear, area);
        
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Delete Student")
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(theme.primary));
            
        let inner_area = block.inner(area);
        frame.render_widget(block, area);
        
        let items = self.students.iter().enumerate().map(|(i, student)| {
            let style = if i == self.selected_index {
                Style::default().fg(theme.highlight)
            } else {
                Style::default().fg(theme.text)
            };
            
            Line::from(Span::styled(&student.username, style))
        }).collect::<Vec<_>>();
        
        let paragraph = Paragraph::new(items)
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left);
            
        frame.render_widget(paragraph, inner_area);
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}