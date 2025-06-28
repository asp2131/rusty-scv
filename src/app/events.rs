use crate::data::{Class, Student};
use crate::ui::screens::ScreenType;

#[derive(Debug, Clone)]
pub enum AppEvent {
    NavigateToScreen(ScreenType),
    GoBack,
    Quit,
    
    // Loading states
    ShowLoading(String),
    HideLoading,
    
    // Error handling
    ShowError(String),
    ClearError,
    
    // Success messages
    ShowSuccess(String),
    
    // Class management
    SelectClass(Class),
    ClassCreated(Class),
    ClassDeleted(i64),
    
    // Student management
    StudentAdded(Student),
    StudentDeleted(i64),
    
    // Repository operations
    CloneRepositories,
    PullRepositories,
    CleanRepositories,
    
    // GitHub operations
    FetchGitHubActivity,
    RefreshData,
}

pub struct EventHandler;

impl EventHandler {
    pub fn new() -> Self {
        Self
    }
}