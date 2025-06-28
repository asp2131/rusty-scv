use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Terminal,
};
use std::{
    io,
    time::{Duration, Instant},
};
use tokio::time::interval;

use crate::ui::{
    animations::AnimationState,
    components::loading::LoadingWidget,
    layout::ResponsiveLayout,
    screens::{Screen, ScreenType},
    themes::{Theme, THEMES},
};

pub mod config;
pub mod events;
pub mod state;

pub use config::Config;
pub use events::{AppEvent, EventHandler};
pub use state::{AppState, MenuState, NavigationStack};

const FRAME_RATE: u64 = 60; // Target 60 FPS
const FRAME_DURATION: Duration = Duration::from_millis(1000 / FRAME_RATE);

pub struct App {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    state: AppState,
    event_handler: EventHandler,
    animation_state: AnimationState,
    layout: ResponsiveLayout,
    theme: &'static Theme,
    config: Config,
    last_frame: Instant,
    github_token: Option<String>,
    should_quit: bool,
    navigation_stack: NavigationStack,
    current_screen: Box<dyn Screen>,
}

impl App {
    pub async fn new(github_token: Option<String>) -> Result<Self> {
        // Initialize terminal
        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend)?;
        
        // Load configuration
        let config = Config::load().await?;
        
        // Initialize components
        let state = AppState::new().await?;
        let event_handler = EventHandler::new();
        let animation_state = AnimationState::new();
        let layout = ResponsiveLayout::new();
        let theme = &THEMES.neon_night;
        let navigation_stack = NavigationStack::new();
        
        // Create initial screen
        let current_screen = Box::new(crate::ui::screens::main_menu::MainMenuScreen::new());

        Ok(Self {
            terminal,
            state,
            event_handler,
            animation_state,
            layout,
            theme,
            config,
            last_frame: Instant::now(),
            github_token,
            should_quit: false,
            navigation_stack,
            current_screen,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal
        crate::utils::terminal::setup_terminal()?;
        
        // Create frame timer
        let mut frame_timer = interval(FRAME_DURATION);
        
        // Main application loop
        loop {
            if self.should_quit {
                break;
            }

            // Handle events with timeout to maintain frame rate
            if event::poll(FRAME_DURATION / 4)? {
                let event = event::read()?;
                self.handle_terminal_event(event).await?;
            }

            // Update animations and state
            self.update().await?;
            
            // Render frame
            self.render()?;
            
            // Wait for next frame
            frame_timer.tick().await;
        }

        Ok(())
    }

    async fn handle_terminal_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Key(key_event) => {
                self.handle_key_event(key_event).await?;
            },
            Event::Resize(width, height) => {
                self.layout.update_size(width, height);
                self.terminal.resize(Rect::new(0, 0, width, height))?;
            },
            Event::Mouse(_) => {
                // Handle mouse events if needed
            },
            _ => {}
        }
        Ok(())
    }

    async fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        // Global key bindings
        match (key_event.code, key_event.modifiers) {
            (KeyCode::Char('c'), KeyModifiers::CONTROL) | (KeyCode::Char('q'), KeyModifiers::NONE) => {
                self.should_quit = true;
                return Ok(());
            },
            (KeyCode::Esc, KeyModifiers::NONE) => {
                if self.navigation_stack.can_go_back() {
                    self.go_back().await?;
                } else {
                    self.should_quit = true;
                }
                return Ok(());
            },
            _ => {}
        }

        // Let current screen handle the event
        let app_event = self.current_screen.handle_key_event(key_event, &self.state).await?;
        
        match app_event {
            Some(event) => self.handle_app_event(event).await?,
            None => {}
        }

        Ok(())
    }

    async fn handle_app_event(&mut self, event: AppEvent) -> Result<()> {
        match event {
            AppEvent::NavigateToScreen(screen_type) => {
                self.navigate_to_screen(screen_type).await?;
            },
            AppEvent::GoBack => {
                self.go_back().await?;
            },
            AppEvent::Quit => {
                self.should_quit = true;
            },
            AppEvent::ShowLoading(message) => {
                self.state.set_loading(true, message);
            },
            AppEvent::HideLoading => {
                self.state.set_loading(false, String::new());
            },
            AppEvent::ShowError(error) => {
                self.state.set_error(Some(error));
            },
            AppEvent::ClearError => {
                self.state.set_error(None);
            },
            AppEvent::ShowSuccess(message) => {
                // TODO: Implement success message display
                println!("Success: {}", message);
            },
            AppEvent::SelectClass(class) => {
                self.state.current_class = Some(class);
            },
            AppEvent::ClassCreated(_class) => {
                // TODO: Handle class creation
            },
            AppEvent::ClassDeleted(_id) => {
                // TODO: Handle class deletion
            },
            AppEvent::StudentAdded(_student) => {
                // TODO: Handle student addition
            },
            AppEvent::StudentDeleted(_id) => {
                // TODO: Handle student deletion
            },
            AppEvent::CloneRepositories => {
                // TODO: Implement repository cloning
            },
            AppEvent::PullRepositories => {
                // TODO: Implement repository pulling
            },
            AppEvent::CleanRepositories => {
                // TODO: Implement repository cleaning
            },
            AppEvent::FetchGitHubActivity => {
                // TODO: Implement GitHub activity fetching
            },
            AppEvent::RefreshData => {
                // TODO: Implement data refresh
            },
        }
        Ok(())
    }

    async fn navigate_to_screen(&mut self, screen_type: ScreenType) -> Result<()> {
        // Save current screen to navigation stack
        let current_type = self.current_screen.screen_type();
        self.navigation_stack.push(current_type);

        // Create new screen
        self.current_screen = crate::ui::screens::create_screen(screen_type)?;
        
        // Trigger enter animation
        self.animation_state.trigger_transition();
        
        Ok(())
    }

    async fn go_back(&mut self) -> Result<()> {
        if let Some(previous_screen_type) = self.navigation_stack.pop() {
            self.current_screen = crate::ui::screens::create_screen(previous_screen_type)?;
            self.animation_state.trigger_transition();
        }
        Ok(())
    }

    async fn update(&mut self) -> Result<()> {
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_frame);
        self.last_frame = now;

        // Update animations
        self.animation_state.update(delta_time);
        
        // Update current screen
        self.current_screen.update(delta_time, &mut self.state).await?;

        Ok(())
    }

    fn render(&mut self) -> Result<()> {
        let area_size = self.terminal.size()?;
        self.layout.update_size(area_size.width, area_size.height);
        
        let state = &self.state;
        let animation_state = &self.animation_state;
        let theme = self.theme;
        
        self.terminal.draw(|frame| {
            let area = frame.size();
            
            // Render current screen
            self.current_screen.render(frame, area, state, animation_state, theme);
            
            // Render global overlays (loading, errors, etc.)
            Self::render_overlays_static(frame, area, state, animation_state, theme);
        })?;
        
        Ok(())
    }

    fn render_overlays_static(frame: &mut ratatui::Frame<ratatui::backend::CrosstermBackend<std::io::Stdout>>, area: Rect, state: &AppState, animation_state: &AnimationState, theme: &Theme) {
        // Render loading overlay
        if state.is_loading() {
            let loading_area = crate::ui::layout::center_rect(40, 20, area);
            frame.render_widget(Clear, loading_area); // Clear background
            
            let loading_widget = LoadingWidget::new(
                state.loading_message().unwrap_or("Loading..."),
                animation_state,
                theme,
            );
            frame.render_widget(loading_widget, loading_area);
        }

        // Render error overlay
        if let Some(error) = state.error() {
            let error_area = crate::ui::layout::center_rect(60, 30, area);
            frame.render_widget(Clear, error_area);
            
            let error_block = Block::default()
                .title("Error")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.error))
                .title_style(Style::default().fg(theme.error).add_modifier(Modifier::BOLD));
            
            let error_text = Paragraph::new(error)
                .block(error_block)
                .wrap(Wrap { trim: true })
                .style(Style::default().fg(theme.text));
            
            frame.render_widget(error_text, error_area);
        }
    }
}

// Helper function to center a rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}