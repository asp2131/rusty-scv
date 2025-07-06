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

use crate::{
    data::Database,
    ui::{
        animations::AnimationState,
        components::loading::LoadingWidget,
        layout::ResponsiveLayout,
        screens::{Screen, ScreenType, ScreenTypeVariant, ScreenContext}, // Fixed imports
        themes::{Theme, THEMES},
    },
};

pub mod config;
pub mod events;
pub mod state;

pub use config::Config;
pub use events::{AppEvent, EventHandler};
pub use state::{AppState, NavigationStack}; // Removed MenuState as it's unused

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
        // Check if there's an error/success message to dismiss
        if self.state.error().is_some() {
            // Any key dismisses the error message
            self.state.set_error(None);
            return Ok(());
        }

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
                // Check if we need to add context to the screen type
                let screen_type = if screen_type.variant() == &ScreenTypeVariant::ClassManagement {
                    if let Some(class) = &self.state.current_class {
                        screen_type.with_context(ScreenContext::Class(class.clone()))
                    } else {
                        screen_type
                    }
                } else {
                    screen_type
                };
                self.navigate_to_screen(screen_type).await?;
            },
            AppEvent::GoBack => {
                self.go_back().await?;
            },
            AppEvent::Quit => {
                self.should_quit = true;
            },
            AppEvent::ShowLoading(message) => {
                self.state.set_loading(true, message.clone());
                
                // Check if this is a class creation loading event
                if message.starts_with("Creating class '") {
                    // Extract class name from message
                    if let Some(start) = message.find('\'') {
                        if let Some(end) = message[start+1..].find('\'') {
                            let class_name = &message[start+1..start+1+end];
                            
                            // Create the class asynchronously
                            let state = &self.state;
                            let db = &state.database;
                            
                            // Clone what we need for the async block
                            let class_name_clone = class_name.to_string();
                            
                            // Schedule the database operation
                            tokio::spawn(async move {
                                // This will be handled in the next frame
                                // For now, just create the loading state
                            });
                            
                            // Create the class asynchronously
                            match db.create_class(&class_name).await {
                                Ok(class) => {
                                    self.state.set_loading(false, String::new());
                                    self.animation_state.trigger_success_celebration();
                                    
                                    // Navigate back to class selection
                                    self.navigate_to_screen(ScreenType::new(ScreenTypeVariant::ClassSelection)).await?;
                                    
                                    // Show success message (temporarily using error display for visibility)
                                    self.state.set_error(Some(format!("✅ Class '{}' created successfully!", class.name)));
                                }
                                Err(e) => {
                                    self.state.set_loading(false, String::new());
                                    self.state.set_error(Some(format!("Failed to create class: {}", e)));
                                    
                                    // Go back to create class screen
                                    if let Ok(screen) = crate::ui::screens::create_screen(ScreenType::new(ScreenTypeVariant::CreateClass)).await {
                                        self.current_screen = screen;
                                    }
                                }
                            }
                        }
                    }
                }
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
                // Store the selected class in the app state
                self.state.current_class = Some(class.clone());
                
                // Navigate to the class management screen with the selected class
                self.navigate_to_screen(
                    ScreenType::new(ScreenTypeVariant::ClassManagement)
                        .with_context(ScreenContext::Class(class))
                ).await?;
            },
            AppEvent::ClassCreated(class) => {
                // Create the class in the database
                self.state.set_loading(true, format!("Creating class '{}'...", class.name));
                
                match self.state.database.create_class(&class.name).await {
                    Ok(created_class) => {
                        self.state.set_loading(false, String::new());
                        self.animation_state.trigger_success_celebration();
                        
                        // Navigate to class selection
                        self.navigate_to_screen(ScreenType::new(ScreenTypeVariant::ClassSelection)).await?;
                        
                        // Show success message (temporarily using error display)
                        self.state.set_error(Some(format!("✅ Class '{}' created successfully!", created_class.name)));
                        
                        // Clear the message after a delay
                        // TODO: Implement timed message clearing
                    }
                    Err(e) => {
                        self.state.set_loading(false, String::new());
                        self.state.set_error(Some(format!("Failed to create class: {}", e)));
                    }
                }
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
                // Handle refresh based on current screen
                match self.current_screen.screen_type().variant() {
                    ScreenTypeVariant::ClassSelection => {
                        // Refresh classes for the class selection screen
                        match self.state.database.get_classes().await {
                            Ok(classes) => {
                                // Cast to specific screen type to call set_classes
                                if let Some(class_screen) = self.current_screen.as_any_mut().downcast_mut::<crate::ui::screens::class_selection::ClassSelectionScreen>() {
                                    class_screen.set_classes(classes);
                                }
                            }
                            Err(e) => {
                                self.state.set_error(Some(format!("Failed to refresh classes: {}", e)));
                            }
                        }
                    }
                    _ => {
                        // For other screens, just ignore refresh for now
                    }
                }
            },
        }
        Ok(())
    }

    async fn navigate_to_screen(&mut self, screen_type: ScreenType) -> Result<()> {
        self.navigation_stack.push(self.current_screen.screen_type());
        self.current_screen = crate::ui::screens::create_screen(screen_type.clone()).await?;
        self.animation_state.trigger_transition();
        Ok(())
    }
    
    // Also update the go_back method to refresh data when going back
    
    async fn go_back(&mut self) -> Result<()> {
        if let Some(previous_screen_type) = self.navigation_stack.pop() {
            self.current_screen = crate::ui::screens::create_screen(previous_screen_type.clone()).await?;
            self.animation_state.trigger_transition();
            
            // Auto-refresh data when going back to certain screens
            match previous_screen_type.variant() {
                ScreenTypeVariant::ClassSelection => {
                    match self.state.database.get_classes().await {
                        Ok(classes) => {
                            if let Some(class_screen) = self.current_screen.as_any_mut().downcast_mut::<crate::ui::screens::class_selection::ClassSelectionScreen>() {
                                class_screen.set_classes(classes);
                            }
                        }
                        Err(e) => {
                            self.state.set_error(Some(format!("Failed to refresh classes: {}", e)));
                        }
                    }
                },
                _ => {}
            }
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
            
            // Determine if this is a success message (starts with ✅) or error
            let is_success = error.starts_with("✅");
            let title = if is_success { "Success" } else { "Error" };
            let border_color = if is_success { theme.success } else { theme.error };
            
            let error_block = Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title_style(Style::default().fg(border_color).add_modifier(Modifier::BOLD));
            
            let inner_area = error_block.inner(error_area);
            frame.render_widget(error_block, error_area);
            
            // Split area for message and help text
            use ratatui::layout::{Constraint, Direction, Layout};
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(1),     // Message area
                    Constraint::Length(1),  // Help text
                ])
                .split(inner_area);
            
            let error_text = Paragraph::new(error)
                .wrap(Wrap { trim: true })
                .style(Style::default().fg(theme.text));
            
            frame.render_widget(error_text, chunks[0]);
            
            // Add help text
            let help_text = ratatui::text::Line::from(vec![
                ratatui::text::Span::styled("Press ", Style::default().fg(theme.text_secondary)),
                ratatui::text::Span::styled("any key", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                ratatui::text::Span::styled(" to dismiss", Style::default().fg(theme.text_secondary)),
            ]);
            
            let help_paragraph = Paragraph::new(help_text)
                .alignment(ratatui::layout::Alignment::Center);
            
            frame.render_widget(help_paragraph, chunks[1]);
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