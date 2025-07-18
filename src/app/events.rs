use crate::data::{Class, Student};
use crate::ui::screens::ScreenType; // Fixed import - removed unused ScreenTypeVariant and ScreenContext

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
    
    // Individual repo actions
    CloneRepo(String), // github_username
    PullRepo(String), // github_username
    CleanRepo(String), // github_username
    OpenInTerminal(String), // github_username
    
    // Batch repo actions
    CloneAllRepos,
    
    // GitHub operations
    FetchGitHubActivity,
    ShowWeekView,
    ShowLatestActivity,
    RefreshLatestActivity,
    RefreshData,
}

pub struct EventHandler;

impl EventHandler {
    pub fn new() -> Self {
        Self
    }
}