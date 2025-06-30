pub mod dashboard;
pub mod input;
pub mod loading;
pub mod main_menu;
pub mod menu;

// Re-export the menu components that are being used
pub use menu::{AnimatedMenu, MenuBuilder, MenuItem, MenuPresets};