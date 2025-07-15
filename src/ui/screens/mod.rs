pub mod add_students;
pub mod class_management;
pub mod class_selection;
pub mod create_class;
pub mod delete_student;
pub mod main_menu;
pub mod student_management;
pub mod github_activity;
pub mod repo_management;
pub mod week_view;
pub mod latest_activity;

use anyhow::Result;
use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect};
use std::{future::Future, pin::Pin, time::Duration};

use crate::{
    app::{AppEvent, AppState},
    data::{Class, Database, Student},
    ui::{animations::AnimationState, themes::Theme},
};

#[derive(Debug, Clone, PartialEq)]
pub struct ScreenType {
    variant: ScreenTypeVariant,
    context: Option<ScreenContext>,
}

impl ScreenType {
    pub fn new(variant: ScreenTypeVariant) -> Self {
        Self {
            variant,
            context: None,
        }
    }

    pub fn with_context(mut self, context: ScreenContext) -> Self {
        self.context = Some(context);
        self
    }

    pub fn context(&self) -> Option<&ScreenContext> {
        self.context.as_ref()
    }
}

impl From<ScreenTypeVariant> for ScreenType {
    fn from(variant: ScreenTypeVariant) -> Self {
        ScreenType::new(variant)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScreenTypeVariant {
    MainMenu,
    ClassSelection,
    CreateClass,
    ClassManagement,
    StudentManagement,
    AddStudents,
    RemoveStudent,
    RepositoryManagement,
    GitHubActivity,
    WeekView,
    LatestActivity,
    Settings,
    ConfirmDeleteClass,
    DeleteStudent,
}

impl std::fmt::Display for ScreenTypeVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScreenTypeVariant::MainMenu => write!(f, "Main Menu"),
            ScreenTypeVariant::ClassSelection => write!(f, "Class Selection"),
            ScreenTypeVariant::CreateClass => write!(f, "Create Class"),
            ScreenTypeVariant::ClassManagement => write!(f, "Class Management"),
            ScreenTypeVariant::StudentManagement => write!(f, "Student Management"),
            ScreenTypeVariant::AddStudents => write!(f, "Add Students"),
            ScreenTypeVariant::RemoveStudent => write!(f, "Remove Student"),
            ScreenTypeVariant::RepositoryManagement => write!(f, "Repository Management"),
            ScreenTypeVariant::GitHubActivity => write!(f, "GitHub Activity"),
            ScreenTypeVariant::WeekView => write!(f, "Week View"),
            ScreenTypeVariant::LatestActivity => write!(f, "Latest Activity"),
            ScreenTypeVariant::Settings => write!(f, "Settings"),
            ScreenTypeVariant::ConfirmDeleteClass => write!(f, "Confirm Delete Class"),
            ScreenTypeVariant::DeleteStudent => write!(f, "Delete Student"),
        }
    }
}

// Implement PartialEq for ScreenType to compare variants only
impl PartialEq<ScreenTypeVariant> for ScreenType {
    fn eq(&self, other: &ScreenTypeVariant) -> bool {
        &self.variant == other
    }
}

impl ScreenType {
    pub fn variant(&self) -> &ScreenTypeVariant {
        &self.variant
    }
}

pub trait Screen {
    fn screen_type(&self) -> ScreenType;
    
    fn handle_key_event<'a>(&'a mut self, key: KeyEvent, state: &'a AppState) -> Pin<Box<dyn Future<Output = Result<Option<AppEvent>>> + Send + 'a>>;
    
    fn update<'a>(&'a mut self, delta_time: Duration, state: &'a mut AppState) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>;
    
    fn render(&mut self, frame: &mut ratatui::Frame<ratatui::backend::CrosstermBackend<std::io::Stdout>>, area: Rect, state: &AppState, animation_state: &AnimationState, theme: &Theme);
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

// Create a screen with the given type and optional context
pub async fn create_screen(screen_type: ScreenType) -> Result<Box<dyn Screen>> {
    match screen_type.variant() {
        ScreenTypeVariant::MainMenu => Ok(Box::new(main_menu::MainMenuScreen::new())),
        ScreenTypeVariant::ClassSelection => Ok(Box::new(class_selection::ClassSelectionScreen::new())),
        ScreenTypeVariant::ClassManagement => {
            if let Some(ScreenContext::Class(class)) = screen_type.context() {
                return Ok(Box::new(class_management::ClassManagementScreen::new(class.clone())));
            }
            Err(anyhow::anyhow!("ClassManagement screen requires class context"))
        },
        ScreenTypeVariant::StudentManagement => {
            if let Some(ScreenContext::Class(class)) = screen_type.context() {
                return Ok(Box::new(student_management::StudentManagementScreen::new(class.clone())));
            }
            Err(anyhow::anyhow!("StudentManagement screen requires class context"))
        },
        ScreenTypeVariant::AddStudents => {
            if let Some(ScreenContext::Class(class)) = screen_type.context() {
                return Ok(Box::new(add_students::AddStudentsScreen::new(class.clone())));
            }
            Err(anyhow::anyhow!("AddStudents screen requires class context"))
        },
        ScreenTypeVariant::CreateClass => Ok(Box::new(create_class::CreateClassScreen::new())),
        ScreenTypeVariant::DeleteStudent => {
            if let Some(ScreenContext::Class(class)) = screen_type.context() {
                let db = Database::init().await?;
                let students = db.get_students_for_class(class.id).await?;
                return Ok(Box::new(delete_student::DeleteStudentScreen::new(class.clone(), students)));
            }
            Err(anyhow::anyhow!("DeleteStudent screen requires class context"))
        }
        ScreenTypeVariant::GitHubActivity => {
            if let Some(ScreenContext::Class(class)) = screen_type.context() {
                return Ok(Box::new(github_activity::GitHubActivityScreen::new(class.clone())));
            }
            Err(anyhow::anyhow!("GitHubActivity screen requires class context"))
        },
        ScreenTypeVariant::RepositoryManagement => {
            if let Some(ScreenContext::Class(class)) = screen_type.context() {
                let db = Database::init().await?;
                let students = db.get_students_for_class(class.id).await?;
                return Ok(Box::new(repo_management::RepoManagementScreen::new(class.clone(), students)));
            }
            Err(anyhow::anyhow!("RepositoryManagement screen requires class context"))
        },
        ScreenTypeVariant::WeekView => {
            if let Some(ScreenContext::Class(class)) = screen_type.context() {
                let db = Database::init().await?;
                let students = db.get_students_for_class(class.id).await?;
                return Ok(Box::new(week_view::WeekViewScreen::new(class.clone(), students)));
            }
            Err(anyhow::anyhow!("WeekView screen requires class context"))
        },
        ScreenTypeVariant::LatestActivity => {
            if let Some(ScreenContext::Class(class)) = screen_type.context() {
                let db = Database::init().await?;
                let students = db.get_students_for_class(class.id).await?;
                return Ok(Box::new(latest_activity::LatestActivityScreen::new(students)));
            }
            Err(anyhow::anyhow!("LatestActivity screen requires class context"))
        },
        _ => anyhow::bail!("Screen type not implemented: {:?}", screen_type.variant()),
    }
}

// Context for screens that need additional data
#[derive(Debug, Clone, PartialEq)]
pub enum ScreenContext {
    Class(Class),
    Student(Student),
    ClassAndStudent(Class, Student),
}