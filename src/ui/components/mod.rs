pub mod confirmation_dialog;
pub mod dashboard;
pub mod input;
pub mod loading;
pub mod main_menu;
pub mod menu;

// Re-export the components that are being used
pub use menu::{AnimatedMenu, MenuBuilder, MenuItem, MenuPresets};
pub use confirmation_dialog::ConfirmationDialog;