use crate::data::{Database, Class}; // Removed unused Student import
use crate::ui::screens::ScreenType;
use crate::git::GitManager;
use std::path::PathBuf;

pub struct AppState {
    pub database: Database,
    pub git_manager: GitManager,
    pub current_class: Option<Class>,
    pub loading: bool,
    pub loading_message: String,
    pub error: Option<String>,
}

impl AppState {
    pub async fn new() -> anyhow::Result<Self> {
        let database = Database::init().await?;
        
        // Create repos directory in home folder
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let repos_dir = home_dir.join("rusty-scv-repos");
        std::fs::create_dir_all(&repos_dir)?;
        
        let git_manager = GitManager::new(repos_dir);
        
        Ok(Self {
            database,
            git_manager,
            current_class: None,
            loading: false,
            loading_message: String::new(),
            error: None,
        })
    }
    
    // Helper methods
    pub fn set_loading(&mut self, loading: bool, message: String) {
        self.loading = loading;
        self.loading_message = message;
    }
    
    pub fn set_error(&mut self, error: Option<String>) {
        self.error = error;
    }
    
    pub fn is_loading(&self) -> bool {
        self.loading
    }
    
    pub fn loading_message(&self) -> Option<&str> {
        if self.loading {
            Some(&self.loading_message)
        } else {
            None
        }
    }
    
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

pub struct NavigationStack {
    stack: Vec<ScreenType>,
}

impl NavigationStack {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
        }
    }
    
    pub fn push(&mut self, screen_type: ScreenType) {
        self.stack.push(screen_type);
    }
    
    pub fn pop(&mut self) -> Option<ScreenType> {
        self.stack.pop()
    }
    
    pub fn can_go_back(&self) -> bool {
        !self.stack.is_empty()
    }
    
    pub fn clear(&mut self) {
        self.stack.clear();
    }
}