pub mod main_menu;
pub mod class_selection;
pub mod create_class;
// TODO: Implement in next phase
// pub mod class_management;
// pub mod student_management;

use anyhow::Result;
use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect};
use std::{future::Future, pin::Pin, time::Duration};

use crate::{
    app::{AppEvent, AppState},
    data::{Class, Student},
    ui::{animations::AnimationState, themes::Theme},
};

#[derive(Debug, Clone, PartialEq)]
pub enum ScreenType {
    MainMenu,
    ClassSelection,
    CreateClass,
    ClassManagement,
    StudentManagement,
    AddStudents,
    RemoveStudent,
    RepositoryManagement,
    GitHubActivity,
    Settings,
    ConfirmDeleteClass,
}

pub trait Screen {
    fn screen_type(&self) -> ScreenType;
    
    fn handle_key_event(&mut self, key: KeyEvent, state: &AppState) -> Pin<Box<dyn Future<Output = Result<Option<AppEvent>>> + Send + '_>>;
    
    fn update(&mut self, delta_time: Duration, state: &mut AppState) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;
    
    fn render(&mut self, frame: &mut ratatui::Frame<ratatui::backend::CrosstermBackend<std::io::Stdout>>, area: Rect, state: &AppState, animation_state: &AnimationState, theme: &Theme);
}

// Simplified version without context for now
pub fn create_screen(screen_type: ScreenType) -> Result<Box<dyn Screen>> {
    match screen_type {
        ScreenType::MainMenu => Ok(Box::new(main_menu::MainMenuScreen::new())),
        ScreenType::ClassSelection => Ok(Box::new(class_selection::ClassSelectionScreen::new())),
        ScreenType::CreateClass => Ok(Box::new(create_class::CreateClassScreen::new())),
        _ => todo!("Implement remaining screens with context"),
    }
}

// Context for screens that need additional data
#[derive(Debug, Clone)]
pub enum ScreenContext {
    Class(Class),
    Student(Student),
    ClassAndStudent(Class, Student),
}