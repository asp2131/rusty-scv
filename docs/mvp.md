---

## Phase 7: Screen System Expansion (4-5 hours)

### 🎯 Goal
Implement all remaining screens with smooth transitions and animations.

### 📋 Tasks

#### 7.1 Class Selection Screen (1 hour)
Create `src/ui/screens/class_selection.rs`:
```rust
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{Frame, layout::Rect};
use std::time::Duration;

use crate::{
    app::{AppEvent, AppState},
    data::Class,
    ui::{
        animations::AnimationState,
        components::menu::{AnimatedMenu, MenuBuilder, MenuItem},
        screens::{Screen, ScreenType},
        themes::Theme,
    },
};

pub struct ClassSelectionScreen {
    classes: Vec<Class>,
    menu: AnimatedMenu,
    loading: bool,
    needs_refresh: bool,
}

impl ClassSelectionScreen {
    pub fn new() -> Self {
        Self {
            classes: Vec::new(),
            menu: MenuBuilder::new()
                .title("📚 Select a Class")
                .simple_item("Loading...")
                .build(),
            loading: true,
            needs_refresh: true,
        }
    }
    
    async fn refresh_classes(&mut self, state: &AppState) -> Result<()> {
        self.loading = true;
        self.classes = state.database.get_classes().await?;
        self.menu = Self::build_class_menu(&self.classes);
        self.menu.trigger_entrance();
        self.loading = false;
        self.needs_refresh = false;
        Ok(())
    }
    
    fn build_class_menu(classes: &[Class]) -> AnimatedMenu {
        let mut builder = MenuBuilder::new()
            .title("📚 Select a Class");
            
        if classes.is_empty() {
            builder = builder
                .simple_item("No classes found")
                .item(MenuItem::new("Create your first class")
                    .with_description("Start by creating a new class")
                    .with_icon("➕"));
        } else {
            for class in classes {
                builder = builder.item(MenuItem::new(&class.name)
                    .with_description(&format!("Manage class: {}", class.name))
                    .with_icon("📖"));
            }
        }
        
        builder
            .item(MenuItem::new("Create New Class")
                .with_description("Add a new class")
                .with_icon("➕")
                .with_hotkey('c'))
            .item(MenuItem::new("Back")
                .with_description("Return to main menu")
                .with_icon("↩️")
                .with_hotkey('b'))
            .build()
    }
}

impl Screen for ClassSelectionScreen {
    fn screen_type(&self) -> ScreenType {
        ScreenType::ClassSelection
    }

    async fn handle_key_event(&mut self, key: KeyEvent, state: &AppState) -> Result<Option<AppEvent>> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.menu.select_previous();
                Ok(None)
            },
            KeyCode::Down | KeyCode::Char('j') => {
                self.menu.select_next();
                Ok(None)
            },
            KeyCode::Enter | KeyCode::Char(' ') => {
                if let Some(item) = self.menu.selected_item() {
                    match item.title.as_str() {
                        "Create New Class" => {
                            Ok(Some(AppEvent::NavigateToScreen(ScreenType::CreateClass)))
                        },
                        "Back" => {
                            Ok(Some(AppEvent::GoBack))
                        },
                        title if !self.classes.is_empty() => {
                            // Find the selected class
                            if let Some(class) = self.classes.iter().find(|c| c.name == title) {
                                Ok(Some(AppEvent::SelectClass(class.clone())))
                            } else {
                                Ok(None)
                            }
                        },
                        _ => Ok(None),
                    }
                } else {
                    Ok(None)
                }
            },
            KeyCode::Char('c') => {
                Ok(Some(AppEvent::NavigateToScreen(ScreenType::CreateClass)))
            },
            KeyCode::Char('r') => {
                // Refresh classes
                self.needs_refresh = true;
                Ok(None)
            },
            _ => Ok(None),
        }
    }

    async fn update(&mut self, delta_time: Duration, state: &mut AppState) -> Result<()> {
        if self.needs_refresh {
            self.refresh_classes(state).await?;
        }
        
        self.menu.update(delta_time, &AnimationState::new());
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, _state: &AppState, _animation_state: &AnimationState, _theme: &Theme) {
        if self.loading {
            // Render loading state
            let loading_widget = crate::ui::components::loading::LoadingPresets::initializing(_theme);
            frame.render_widget(&mut loading_widget, area);
        } else {
            // Center the menu
            let menu_area = crate::ui::layout::center_rect(60, 80, area);
            frame.render_widget(&mut self.menu, menu_area);
        }
    }
}
```

#### 7.2 Create Class Screen (45 minutes)
Create `src/ui/screens/create_class.rs`:
```rust
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame, 
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};
use std::time::Duration;

use crate::{
    app::{AppEvent, AppState},
    ui::{
        animations::AnimationState,
        components::input::AnimatedInput,
        screens::{Screen, ScreenType},
        themes::Theme,
    },
};

pub struct CreateClassScreen {
    input: AnimatedInput,
    error: Option<String>,
    creating: bool,
}

impl CreateClassScreen {
    pub fn new() -> Self {
        let mut input = AnimatedInput::new("Class Name");
        input.set_placeholder("Enter class name (e.g., 'CS101 Fall 2024')");
        input.focus();
        
        Self {
            input,
            error: None,
            creating: false,
        }
    }
}

impl Screen for CreateClassScreen {
    fn screen_type(&self) -> ScreenType {
        ScreenType::CreateClass
    }

    async fn handle_key_event(&mut self, key: KeyEvent, state: &AppState) -> Result<Option<AppEvent>> {
        if self.creating {
            return Ok(None); // Ignore input while creating
        }

        match key.code {
            KeyCode::Enter => {
                let class_name = self.input.value().trim();
                if class_name.is_empty() {
                    self.error = Some("Class name cannot be empty".to_string());
                    return Ok(None);
                }

                self.creating = true;
                self.error = None;

                // Create the class
                match state.database.create_class(class_name).await {
                    Ok(_class) => {
                        Ok(Some(AppEvent::ShowSuccess(format!("Created class: {}", class_name))))
                    },
                    Err(e) => {
                        self.creating = false;
                        self.error = Some(format!("Failed to create class: {}", e));
                        Ok(None)
                    }
                }
            },
            KeyCode::Esc => {
                Ok(Some(AppEvent::GoBack))
            },
            _ => {
                self.input.handle_key_event(key);
                self.error = None; // Clear error on new input
                Ok(None)
            }
        }
    }

    async fn update(&mut self, delta_time: Duration, _state: &mut AppState) -> Result<()> {
        self.input.update(delta_time);
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, _state: &AppState, _animation_state: &AnimationState, theme: &Theme) {
        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Length(3),  // Input
                Constraint::Length(2),  // Error/Help
                Constraint::Min(0),     // Remaining
            ])
            .split(area);

        // Title
        let title = Paragraph::new("📚 Create New Class")
            .style(theme.primary_text())
            .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // Input field
        frame.render_widget(&mut self.input, chunks[1]);

        // Error or help text
        if let Some(ref error) = self.error {
            let error_text = Paragraph::new(error.as_str())
                .style(theme.error_text())
                .alignment(Alignment::Center);
            frame.render_widget(error_text, chunks[2]);
        } else if !self.creating {
            let help_text = Paragraph::new("Press Enter to create • Esc to cancel")
                .style(theme.secondary_text())
                .alignment(Alignment::Center);
            frame.render_widget(help_text, chunks[2]);
        }

        // Loading overlay
        if self.creating {
            let loading_area = crate::ui::layout::center_rect(40, 20, area);
            frame.render_widget(Clear, loading_area);
            
            let mut loading = crate::ui::components::loading::LoadingPresets::initializing(theme);
            frame.render_widget(&mut loading, loading_area);
        }
    }
}
```

#### 7.3 Update App Event System (30 minutes)
Add new events to `src/app/events.rs`:
```rust
use crate::data::Class;

#[derive(Debug, Clone)]
pub enum AppEvent {
    NavigateToScreen(ScreenType),
    GoBack,
    Quit,
    
#### 7.3 Update App Event System (30 minutes)
Add new events to `src/app/events.rs`:
```rust
use crate::data::Class;

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
```

#### 7.4 Class Management Screen (1.5 hours)
Create `src/ui/screens/class_management.rs`:
```rust
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{Frame, layout::Rect};
use std::time::Duration;

use crate::{
    app::{AppEvent, AppState},
    data::{Class, Student},
    ui::{
        animations::AnimationState,
        components::menu::{AnimatedMenu, MenuPresets},
        screens::{Screen, ScreenType},
        themes::Theme,
    },
};

pub struct ClassManagementScreen {
    class: Class,
    menu: AnimatedMenu,
    student_count: i64,
    loading: bool,
}

impl ClassManagementScreen {
    pub fn new(class: Class) -> Self {
        let menu = MenuPresets::class_management(&class.name);
        
        Self {
            class,
            menu,
            student_count: 0,
            loading: false,
        }
    }
    
    async fn refresh_data(&mut self, state: &AppState) -> Result<()> {
        self.student_count = state.database.get_student_count_for_class(self.class.id).await?;
        Ok(())
    }
}

impl Screen for ClassManagementScreen {
    fn screen_type(&self) -> ScreenType {
        ScreenType::ClassManagement
    }

    async fn handle_key_event(&mut self, key: KeyEvent, _state: &AppState) -> Result<Option<AppEvent>> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.menu.select_previous();
                Ok(None)
            },
            KeyCode::Down | KeyCode::Char('j') => {
                self.menu.select_next();
                Ok(None)
            },
            KeyCode::Enter | KeyCode::Char(' ') => {
                if let Some(item) = self.menu.selected_item() {
                    match item.title.as_str() {
                        "Manage Students" => {
                            Ok(Some(AppEvent::NavigateToScreen(ScreenType::StudentManagement)))
                        },
                        "Manage Repositories" => {
                            Ok(Some(AppEvent::NavigateToScreen(ScreenType::RepositoryManagement)))
                        },
                        "View GitHub Activity" => {
                            Ok(Some(AppEvent::NavigateToScreen(ScreenType::GitHubActivity)))
                        },
                        "Delete Class" => {
                            Ok(Some(AppEvent::NavigateToScreen(ScreenType::ConfirmDeleteClass)))
                        },
                        "Back" => {
                            Ok(Some(AppEvent::GoBack))
                        },
                        _ => Ok(None),
                    }
                } else {
                    Ok(None)
                }
            },
            // Hotkeys
            KeyCode::Char('s') => {
                Ok(Some(AppEvent::NavigateToScreen(ScreenType::StudentManagement)))
            },
            KeyCode::Char('r') => {
                Ok(Some(AppEvent::NavigateToScreen(ScreenType::RepositoryManagement)))
            },
            KeyCode::Char('a') => {
                Ok(Some(AppEvent::NavigateToScreen(ScreenType::GitHubActivity)))
            },
            KeyCode::Char('d') => {
                Ok(Some(AppEvent::NavigateToScreen(ScreenType::ConfirmDeleteClass)))
            },
            _ => Ok(None),
        }
    }

    async fn update(&mut self, delta_time: Duration, state: &mut AppState) -> Result<()> {
        // Refresh data periodically
        if !self.loading {
            self.refresh_data(state).await?;
        }
        
        self.menu.update(delta_time, &AnimationState::new());
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, _state: &AppState, _animation_state: &AnimationState, _theme: &Theme) {
        // Add class info at the top
        let info_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 3,
        };
        
        let class_info = format!("📚 {} • {} students", self.class.name, self.student_count);
        let info_paragraph = ratatui::widgets::Paragraph::new(class_info)
            .style(_theme.secondary_text())
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(info_paragraph, info_area);
        
        // Render menu below info
        let menu_area = Rect {
            x: area.x,
            y: area.y + 3,
            width: area.width,
            height: area.height - 3,
        };
        
        let centered_menu_area = crate::ui::layout::center_rect(60, 80, menu_area);
        frame.render_widget(&mut self.menu, centered_menu_area);
    }
}
```

#### 7.5 Student Management Screen (1.5 hours)
Create `src/ui/screens/student_management.rs`:
```rust
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame, 
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::time::Duration;

use crate::{
    app::{AppEvent, AppState},
    data::{Class, Student},
    ui::{
        animations::AnimationState,
        components::menu::{AnimatedMenu, MenuPresets},
        screens::{Screen, ScreenType},
        themes::Theme,
    },
};

pub struct StudentManagementScreen {
    class: Class,
    students: Vec<Student>,
    menu: AnimatedMenu,
    show_student_list: bool,
    loading: bool,
}

impl StudentManagementScreen {
    pub fn new(class: Class) -> Self {
        let menu = MenuPresets::student_management(&class.name);
        
        Self {
            class,
            students: Vec::new(),
            menu,
            show_student_list: true,
            loading: false,
        }
    }
    
    async fn refresh_students(&mut self, state: &AppState) -> Result<()> {
        self.loading = true;
        self.students = state.database.get_students_for_class(self.class.id).await?;
        self.loading = false;
        Ok(())
    }
}

impl Screen for StudentManagementScreen {
    fn screen_type(&self) -> ScreenType {
        ScreenType::StudentManagement
    }

    async fn handle_key_event(&mut self, key: KeyEvent, _state: &AppState) -> Result<Option<AppEvent>> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.menu.select_previous();
                Ok(None)
            },
            KeyCode::Down | KeyCode::Char('j') => {
                self.menu.select_next();
                Ok(None)
            },
            KeyCode::Enter | KeyCode::Char(' ') => {
                if let Some(item) = self.menu.selected_item() {
                    match item.title.as_str() {
                        "Add Students" => {
                            Ok(Some(AppEvent::NavigateToScreen(ScreenType::AddStudents)))
                        },
                        "Remove Student" => {
                            if self.students.is_empty() {
                                Ok(Some(AppEvent::ShowError("No students to remove".to_string())))
                            } else {
                                Ok(Some(AppEvent::NavigateToScreen(ScreenType::RemoveStudent)))
                            }
                        },
                        "View Student List" => {
                            self.show_student_list = !self.show_student_list;
                            Ok(None)
                        },
                        "Back" => {
                            Ok(Some(AppEvent::GoBack))
                        },
                        _ => Ok(None),
                    }
                } else {
                    Ok(None)
                }
            },
            KeyCode::Char('a') => {
                Ok(Some(AppEvent::NavigateToScreen(ScreenType::AddStudents)))
            },
            KeyCode::Char('r') => {
                if !self.students.is_empty() {
                    Ok(Some(AppEvent::NavigateToScreen(ScreenType::RemoveStudent)))
                } else {
                    Ok(None)
                }
            },
            KeyCode::Char('v') => {
                self.show_student_list = !self.show_student_list;
                Ok(None)
            },
            KeyCode::F5 => {
                // Refresh student list
                Ok(Some(AppEvent::RefreshData))
            },
            _ => Ok(None),
        }
    }

    async fn update(&mut self, delta_time: Duration, state: &mut AppState) -> Result<()> {
        // Refresh students on first load
        if self.students.is_empty() && !self.loading {
            self.refresh_students(state).await?;
        }
        
        self.menu.update(delta_time, &AnimationState::new());
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, _state: &AppState, _animation_state: &AnimationState, theme: &Theme) {
        if self.show_student_list && !self.students.is_empty() {
            // Split screen: menu on left, student list on right
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area);
            
            // Render menu
            frame.render_widget(&mut self.menu, chunks[0]);
            
            // Render student list
            self.render_student_list(frame, chunks[1], theme);
        } else {
            // Full screen menu
            let centered_area = crate::ui::layout::center_rect(60, 80, area);
            frame.render_widget(&mut self.menu, centered_area);
        }
    }
}

impl StudentManagementScreen {
    fn render_student_list(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let block = Block::default()
            .title(format!("👥 Students ({})", self.students.len()))
            .borders(Borders::ALL)
            .border_style(theme.border_style())
            .title_style(theme.primary_text());
        
        if self.students.is_empty() {
            let empty_msg = Paragraph::new("No students in this class")
                .style(theme.secondary_text())
                .block(block)
                .alignment(ratatui::layout::Alignment::Center);
            frame.render_widget(empty_msg, area);
        } else {
            let items: Vec<ListItem> = self.students
                .iter()
                .enumerate()
                .map(|(i, student)| {
                    let content = format!("{}. {} ({})", i + 1, student.username, student.github_username);
                    ListItem::new(content).style(theme.secondary_text())
                })
                .collect();
            
            let list = List::new(items)
                .block(block)
                .style(theme.secondary_text());
            
            frame.render_widget(list, area);
        }
    }
}
```

#### 7.6 Screen Factory Update (15 minutes)
Update `src/ui/screens/mod.rs` to handle all new screens:
```rust
pub mod main_menu;
pub mod class_selection;
pub mod create_class;
pub mod class_management;
pub mod student_management;
// Add more as you create them

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

pub fn create_screen(screen_type: ScreenType, context: Option<ScreenContext>) -> Result<Box<dyn Screen>> {
    match screen_type {
        ScreenType::MainMenu => Ok(Box::new(main_menu::MainMenuScreen::new())),
        ScreenType::ClassSelection => Ok(Box::new(class_selection::ClassSelectionScreen::new())),
        ScreenType::CreateClass => Ok(Box::new(create_class::CreateClassScreen::new())),
        ScreenType::ClassManagement => {
            if let Some(ScreenContext::Class(class)) = context {
                Ok(Box::new(class_management::ClassManagementScreen::new(class)))
            } else {
                Err(anyhow::anyhow!("Class context required for ClassManagement screen"))
            }
        },
        ScreenType::StudentManagement => {
            if let Some(ScreenContext::Class(class)) = context {
                Ok(Box::new(student_management::StudentManagementScreen::new(class)))
            } else {
                Err(anyhow::anyhow!("Class context required for StudentManagement screen"))
            }
        },
        _ => todo!("Implement remaining screens"),
    }
}

// Context for screens that need additional data
#[derive(Debug, Clone)]
pub enum ScreenContext {
    Class(Class),
    Student(Student),
    ClassAndStudent(Class, Student),
}
```

---

## Phase 8: GitHub Integration (2-3 hours)

### 🎯 Goal
Port your GitHub API operations to Rust with enhanced error handling and caching.

### 📋 Tasks

#### 8.1 GitHub Client Setup (45 minutes)
Create `src/data/github.rs`:
```rust
use anyhow::{anyhow, Result};
use reqwest::{Client, header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT}};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tokio::time::{Duration, Instant};

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubCommit {
    pub sha: String,
    pub commit: CommitDetails,
    pub author: Option<GitHubUser>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommitDetails {
    pub author: CommitAuthor,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommitAuthor {
    pub date: DateTime<Utc>,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub avatar_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubRepository {
    pub name: String,
    pub full_name: String,
    pub private: bool,
    pub updated_at: DateTime<Utc>,
}

pub struct GitHubClient {
    client: Client,
    token: Option<String>,
    rate_limit_cache: HashMap<String, Instant>,
}

impl GitHubClient {
    pub fn new(token: Option<String>) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("scv-rust/1.0"));
        
        if let Some(ref token) = token {
            let auth_value = HeaderValue::from_str(&format!("token {}", token))?;
            headers.insert(AUTHORIZATION, auth_value);
        }
        
        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .build()?;
        
        Ok(Self {
            client,
            token,
            rate_limit_cache: HashMap::new(),
        })
    }
    
    pub async fn get_latest_commit(&mut self, username: &str, repo: &str) -> Result<Option<GitHubCommit>> {
        let url = format!("https://api.github.com/repos/{}/{}/commits?per_page=1", username, repo);
        
        // Check rate limiting
        self.check_rate_limit(&url).await?;
        
        let response = self.client.get(&url).send().await?;
        
        if response.status() == 404 {
            return Ok(None); // Repository not found
        }
        
        if !response.status().is_success() {
            return Err(anyhow!("GitHub API error: {}", response.status()));
        }
        
        let commits: Vec<GitHubCommit> = response.json().await?;
        Ok(commits.into_iter().next())
    }
    
    pub async fn get_commits_in_range(
        &mut self,
        username: &str,
        repo: &str,
        since: DateTime<Utc>,
        until: DateTime<Utc>,
    ) -> Result<Vec<GitHubCommit>> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/commits?since={}&until={}&per_page=100",
            username,
            repo,
            since.to_rfc3339(),
            until.to_rfc3339()
        );
        
        self.check_rate_limit(&url).await?;
        
        let response = self.client.get(&url).send().await?;
        
        if response.status() == 404 {
            return Ok(Vec::new()); // Repository not found
        }
        
        if !response.status().is_success() {
            return Err(anyhow!("GitHub API error: {}", response.status()));
        }
        
        let commits: Vec<GitHubCommit> = response.json().await?;
        Ok(commits)
    }
    
    pub async fn check_repository_exists(&mut self, username: &str, repo: &str) -> Result<bool> {
        let url = format!("https://api.github.com/repos/{}/{}", username, repo);
        
        self.check_rate_limit(&url).await?;
        
        let response = self.client.head(&url).send().await?;
        Ok(response.status().is_success())
    }
    
    async fn check_rate_limit(&mut self, url: &str) -> Result<()> {
        // Simple rate limiting: 1 request per second per endpoint
        if let Some(&last_request) = self.rate_limit_cache.get(url) {
            let elapsed = last_request.elapsed();
            if elapsed < Duration::from_secs(1) {
                tokio::time::sleep(Duration::from_secs(1) - elapsed).await;
            }
        }
        
        self.rate_limit_cache.insert(url.to_string(), Instant::now());
        Ok(())
    }
}

// Helper functions for activity analysis
pub fn analyze_commit_activity(commits: &[GitHubCommit]) -> ActivitySummary {
    let mut summary = ActivitySummary::default();
    
    let now = Utc::now();
    let week_ago = now - chrono::Duration::days(7);
    let month_ago = now - chrono::Duration::days(30);
    
    for commit in commits {
        let commit_date = commit.commit.author.date;
        
        if commit_date > week_ago {
            summary.commits_this_week += 1;
        }
        
        if commit_date > month_ago {
            summary.commits_this_month += 1;
        }
        
        summary.total_commits += 1;
        
        if summary.latest_commit.is_none() || commit_date > summary.latest_commit.unwrap() {
            summary.latest_commit = Some(commit_date);
        }
    }
    
    summary
}

#[derive(Debug, Default)]
pub struct ActivitySummary {
    pub total_commits: usize,
    pub commits_this_week: usize,
    pub commits_this_month: usize,
    pub latest_commit: Option<DateTime<Utc>>,
}
```

#### 8.2 Activity Analysis Functions (45 minutes)
Add activity analysis functions to handle the data processing for GitHub activity visualization.

#### 8.3 Error Handling and Retry Logic (30 minutes)
Implement robust error handling for network issues, rate limiting, and API errors.

#### 8.4 Testing GitHub Integration (45 minutes)
Create tests to verify GitHub API integration works correctly with real and mock data.

---

## Quick Win Milestones 🎯

After each phase, you should achieve these milestones:

### Phase 6 Milestone: "Data Layer Working"
- ✅ Fresh SQLite database created in `~/.scv-rust/`
- ✅ Can create classes and add students
- ✅ All CRUD operations working
- ✅ Database persists between runs

### Phase 7 Milestone: "Navigation Paradise"
- ✅ Smooth transitions between all screens
- ✅ Can create classes via beautiful animated interface
- ✅ Can manage students with live lists
- ✅ All animations working smoothly
- ✅ Error handling with elegant overlays

### Phase 8 Milestone: "GitHub Connected"
- ✅ Live GitHub data integration
- ✅ Real commit information displayed
- ✅ Rate limiting handled gracefully
- ✅ Network errors handled elegantly

## 🚀 Estimated Timeline

- **Weekend 1**: Phases 6-7 (Fresh data layer + Core screens)
- **Weekend 2**: Phases 8-9 (GitHub + Git operations)  
- **Weekend 3**: Phases 10-11 (Advanced UI + Activity viz)
- **Weekend 4**: Phases 12-13 (Polish + Performance)

Each phase builds on the previous, so you can stop at any milestone and have a working application that's better than the original!# Next Steps - Complete Rewrite Roadmap

This comprehensive guide outlines the step-by-step approach to complete the Student Code Viewer Rust rewrite, transforming it from a basic animated menu into a fully-featured terminal application.

## 🎯 Project Status Overview

After completing the Getting Started guide, you should have:
- ✅ Animated main menu with particle effects
- ✅ Theme system with 5 color schemes
- ✅ Animation framework with 60 FPS rendering
- ✅ Responsive layout system
- ✅ Basic navigation infrastructure

🎉 **Great news!** You won't need to migrate any data from your Go application. The Rust version will use a fresh SQLite database stored in `~/.scv-rust/` (separate from your Go version in `~/.scv/`). This means you can run both versions side-by-side during development!

## 📊 Roadmap Overview

| Phase | Focus Area | Duration | Complexity | Priority |
|-------|------------|----------|------------|----------|
| 6 | Fresh Data Layer | 2-3 hours | Low | High |
| 7 | Screen System Expansion | 4-5 hours | Medium | High |
| 8 | GitHub Integration | 2-3 hours | Medium | High |
| 9 | Git Operations | 2-3 hours | Low | High |
| 10 | Advanced UI Components | 3-4 hours | High | Medium |
| 11 | GitHub Activity Visualization | 4-5 hours | High | Medium |
| 12 | Enhanced Features | 3-4 hours | Medium | Low |
| 13 | Polish & Performance | 2-3 hours | Medium | Low |

**Total Estimated Time: 23-33 hours**

---

## Phase 6: Fresh Data Layer Implementation (2-3 hours)

### 🎯 Goal
Create a brand new SQLite database for the Rust application using `sqlx`. No migration needed - start fresh!

### 📋 Tasks

#### 6.1 Database Models (30 minutes)
Create `src/data/models.rs`:
```rust
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Class {
    pub id: i64,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Student {
    pub id: i64,
    pub class_id: i64,
    pub username: String,
    pub github_username: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentWithClass {
    pub student: Student,
    pub class: Class,
}

impl Class {
    pub fn new(name: String) -> Self {
        Self {
            id: 0, // Will be set by database
            name,
            created_at: Utc::now(),
        }
    }
}

impl Student {
    pub fn new(class_id: i64, username: String) -> Self {
        Self {
            id: 0, // Will be set by database
            class_id,
            github_username: username.clone(),
            username,
            created_at: Utc::now(),
        }
    }
}
```

#### 6.2 Database Operations (90 minutes)
Create `src/data/database.rs`:
```rust
use anyhow::Result;
use sqlx::{SqlitePool, migrate::MigrateDatabase, Sqlite, Row};
use std::path::PathBuf;
use dirs::home_dir;

use super::models::{Class, Student};

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn init() -> Result<Self> {
        let db_path = get_database_path()?;
        
        // Create database if it doesn't exist
        if !Sqlite::database_exists(&db_path).await.unwrap_or(false) {
            Sqlite::create_database(&db_path).await?;
        }
        
        let pool = SqlitePool::connect(&db_path).await?;
        
        // Create tables if they don't exist
        Self::create_tables(&pool).await?;
        
        Ok(Self { pool })
    }
    
    async fn create_tables(pool: &SqlitePool) -> Result<()> {
        // Create classes table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS classes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(pool)
        .await?;
        
        // Create students table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS students (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                class_id INTEGER NOT NULL,
                username TEXT NOT NULL,
                github_username TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (class_id) REFERENCES classes (id) ON DELETE CASCADE,
                UNIQUE(class_id, username)
            )
            "#
        )
        .execute(pool)
        .await?;
        
        // Create indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_students_class_id ON students(class_id)")
            .execute(pool)
            .await?;
            
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_students_username ON students(username)")
            .execute(pool)
            .await?;
        
        Ok(())
    }
    
    // ===== CLASS OPERATIONS =====
    
    pub async fn create_class(&self, name: &str) -> Result<Class> {
        let row = sqlx::query(
            "INSERT INTO classes (name, created_at) VALUES (?, CURRENT_TIMESTAMP) RETURNING id, name, created_at"
        )
        .bind(name)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(Class {
            id: row.get("id"),
            name: row.get("name"),
            created_at: row.get("created_at"),
        })
    }
    
    pub async fn get_classes(&self) -> Result<Vec<Class>> {
        let rows = sqlx::query("SELECT id, name, created_at FROM classes ORDER BY name")
            .fetch_all(&self.pool)
            .await?;
            
        let classes = rows.into_iter().map(|row| Class {
            id: row.get("id"),
            name: row.get("name"),
            created_at: row.get("created_at"),
        }).collect();
        
        Ok(classes)
    }
    
    pub async fn get_class_by_id(&self, id: i64) -> Result<Option<Class>> {
        let row = sqlx::query("SELECT id, name, created_at FROM classes WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
            
        match row {
            Some(row) => Ok(Some(Class {
                id: row.get("id"),
                name: row.get("name"),
                created_at: row.get("created_at"),
            })),
            None => Ok(None),
        }
    }
    
    pub async fn delete_class(&self, id: i64) -> Result<bool> {
        let result = sqlx::query("DELETE FROM classes WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(result.rows_affected() > 0)
    }
    
    // ===== STUDENT OPERATIONS =====
    
    pub async fn add_student(&self, class_id: i64, username: &str) -> Result<Student> {
        let row = sqlx::query(
            "INSERT INTO students (class_id, username, github_username, created_at) 
             VALUES (?, ?, ?, CURRENT_TIMESTAMP) 
             RETURNING id, class_id, username, github_username, created_at"
        )
        .bind(class_id)
        .bind(username)
        .bind(username) // Use same username for GitHub
        .fetch_one(&self.pool)
        .await?;
        
        Ok(Student {
            id: row.get("id"),
            class_id: row.get("class_id"),
            username: row.get("username"),
            github_username: row.get("github_username"),
            created_at: row.get("created_at"),
        })
    }
    
    pub async fn get_students_for_class(&self, class_id: i64) -> Result<Vec<Student>> {
        let rows = sqlx::query(
            "SELECT id, class_id, username, github_username, created_at 
             FROM students WHERE class_id = ? ORDER BY username"
        )
        .bind(class_id)
        .fetch_all(&self.pool)
        .await?;
        
        let students = rows.into_iter().map(|row| Student {
            id: row.get("id"),
            class_id: row.get("class_id"),
            username: row.get("username"),
            github_username: row.get("github_username"),
            created_at: row.get("created_at"),
        }).collect();
        
        Ok(students)
    }
    
    pub async fn delete_student(&self, id: i64) -> Result<bool> {
        let result = sqlx::query("DELETE FROM students WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(result.rows_affected() > 0)
    }
    
    pub async fn get_student_count_for_class(&self, class_id: i64) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM students WHERE class_id = ?")
            .bind(class_id)
            .fetch_one(&self.pool)
            .await?;
        
        Ok(row.get("count"))
    }
}

fn get_database_path() -> Result<String> {
    let home = home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let scv_dir = home.join(".scv-rust"); // Different from Go version
    std::fs::create_dir_all(&scv_dir)?;
    
    let db_path = scv_dir.join("scv.db");
    Ok(format!("sqlite://{}", db_path.display()))
}
```

#### 6.3 Database Module Setup (15 minutes)
Update `src/data/mod.rs`:
```rust
pub mod database;
pub mod models;

pub use database::Database;
pub use models::{Class, Student, StudentWithClass};
```

#### 6.4 Integration with App State (30 minutes)
Update your app state to include the database:

**`src/app/state.rs`**
```rust
use crate::data::{Database, Class, Student};

pub struct AppState {
    pub database: Database,
    pub current_class: Option<Class>,
    pub loading: bool,
    pub loading_message: String,
    pub error: Option<String>,
    // ... other state fields
}

impl AppState {
    pub async fn new() -> anyhow::Result<Self> {
        let database = Database::init().await?;
        
        Ok(Self {
            database,
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
```

#### 6.5 Testing the Database (15 minutes)
Create a simple test to verify everything works:

**`src/data/database.rs` (add to the end)**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_database_operations() -> Result<()> {
        let db = Database::init().await?;
        
        // Test class creation
        let class = db.create_class("Test Class").await?;
        assert_eq!(class.name, "Test Class");
        
        // Test student creation
        let student = db.add_student(class.id, "testuser").await?;
        assert_eq!(student.username, "testuser");
        assert_eq!(student.class_id, class.id);
        
        // Test getting students
        let students = db.get_students_for_class(class.id).await?;
        assert_eq!(students.len(), 1);
        
        // Test cleanup
        db.delete_student(student.id).await?;
        db.delete_class(class.id).await?;
        
        Ok(())
    }
}
```

Run the test:
```bash
cargo test test_database_operations
```

---

## Phase 7: Screen System Expansion (4-5 hours)

### 🎯 Goal
Implement all remaining screens with smooth transitions and animations.

### 📋 Tasks

#### 7.1 Class Selection Screen (1 hour)
Create `src/ui/screens/class_selection.rs`:
```rust
pub struct ClassSelectionScreen {
    classes: Vec<Class>,
    menu: AnimatedMenu,
    loading: bool,
}

impl ClassSelectionScreen {
    pub async fn new(database: &Database) -> Result<Self> {
        let classes = database.get_classes().await?;
        let menu = Self::build_class_menu(&classes);
        
        Ok(Self {
            classes,
            menu,
            loading: false,
        })
    }
    
    fn build_class_menu(classes: &[Class]) -> AnimatedMenu {
        let mut builder = MenuBuilder::new()
            .title("📚 Select a Class");
            
        for class in classes {
            builder = builder.item(MenuItem::new(&class.name)
                .with_description(&format!("Manage class: {}", class.name))
                .with_icon("📖"));
        }
        
        builder.item(MenuItem::new("Back")
            .with_description("Return to main menu")
            .with_icon("↩️"))
            .build()
    }
}

impl Screen for ClassSelectionScreen {
    // Implementation follows main menu pattern
}
```

#### 7.2 Class Management Screen (1 hour)
Implement the main class management interface with options


#### 7.3 Student Management Screen (1 hour)
Implement the main student management interface with the following options:
- Add Student
- Remove Student
- View Student

#### 7.4 Done! 

