use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{Frame, layout::Rect};
use std::{future::Future, pin::Pin, time::Duration};

use crate::{
    app::{AppEvent, AppState},
    ui::{
        animations::AnimationState,
        components::menu::{AnimatedMenu, MenuBuilder, MenuItem, MenuPresets},
        screens::{Screen, ScreenType, ScreenTypeVariant},
        themes::Theme,
    },
};

pub struct MainMenuScreen {
    menu: AnimatedMenu,
}

impl MainMenuScreen {
    pub fn new() -> Self {
        let mut menu = MenuPresets::main_menu();
        menu.trigger_entrance();
        
        Self {
            menu,
        }
    }
}

impl Screen for MainMenuScreen {
    fn screen_type(&self) -> ScreenType {
        ScreenType::new(ScreenTypeVariant::MainMenu)
    }

    fn handle_key_event(&mut self, key: KeyEvent, _state: &AppState) -> Pin<Box<dyn Future<Output = Result<Option<AppEvent>>> + Send + '_>> {
        let result = match key.code {
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
                        "Manage Classes" => {
                            Ok(Some(AppEvent::NavigateToScreen(ScreenType::new(ScreenTypeVariant::ClassSelection))))
                        },
                        "Create Class" => {
                            Ok(Some(AppEvent::NavigateToScreen(ScreenType::new(ScreenTypeVariant::CreateClass))))
                        },
                        "Settings" => {
                            Ok(Some(AppEvent::NavigateToScreen(ScreenType::new(ScreenTypeVariant::Settings))))
                        },
                        "Quit" => {
                            Ok(Some(AppEvent::Quit))
                        },
                        _ => Ok(None),
                    }
                } else {
                    Ok(None)
                }
            },
            // Hotkeys
            KeyCode::Char('m') => {
                Ok(Some(AppEvent::NavigateToScreen(ScreenType::new(ScreenTypeVariant::ClassSelection))))
            },
            KeyCode::Char('c') => {
                Ok(Some(AppEvent::NavigateToScreen(ScreenType::new(ScreenTypeVariant::CreateClass))))
            },
            KeyCode::Char('s') => {
                Ok(Some(AppEvent::NavigateToScreen(ScreenType::new(ScreenTypeVariant::Settings))))
            },
            KeyCode::Char('q') => {
                Ok(Some(AppEvent::Quit))
            },
            _ => Ok(None),
        };
        Box::pin(async move { result })
    }

    fn update(&mut self, delta_time: Duration, _state: &mut AppState) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        self.menu.update(delta_time, &AnimationState::new());
        Box::pin(async move { Ok(()) })
    }

    fn render(&mut self, frame: &mut Frame<ratatui::backend::CrosstermBackend<std::io::Stdout>>, area: Rect, _state: &AppState, _animation_state: &AnimationState, _theme: &Theme) {
        // Center the menu
        let menu_area = crate::ui::layout::center_rect(60, 80, area);
        frame.render_widget(&mut self.menu, menu_area);
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}