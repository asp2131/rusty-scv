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

    fn handle_key_event(&mut self, key: KeyEvent, _state: &AppState) -> Pin<Box<dyn Future<Output = Result<Option<AppEvent>>> + Send + '_>> {
        let result =
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
                        "Create New Class" | "Create your first class" => {
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
        };
        Box::pin(async move { result })
    }

    fn update(&mut self, delta_time: Duration, state: &mut AppState) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        let needs_refresh = self.needs_refresh;
        if needs_refresh {
            self.needs_refresh = false; // Reset flag
            // TODO: Implement async refresh
        }
        
        self.menu.update(delta_time, &AnimationState::new());
        Box::pin(async move { Ok(()) })
    }

    fn render(&mut self, frame: &mut ratatui::Frame<ratatui::backend::CrosstermBackend<std::io::Stdout>>, area: Rect, _state: &AppState, _animation_state: &AnimationState, theme: &Theme) {
        if self.loading {
            // Render loading state
            let loading_widget = crate::ui::components::loading::LoadingPresets::initializing(theme);
            frame.render_widget(loading_widget, area);
        } else {
            // Center the menu
            let menu_area = crate::ui::layout::center_rect(60, 80, area);
            frame.render_widget(&mut self.menu, menu_area);
        }
    }
}