use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct ResponsiveLayout {
    width: u16,
    height: u16,
}

impl ResponsiveLayout {
    pub fn new() -> Self {
        Self {
            width: 80,
            height: 24,
        }
    }
    
    pub fn update_size(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
    }
    
    pub fn is_small_screen(&self) -> bool {
        self.width < 80 || self.height < 24
    }
    
    pub fn is_large_screen(&self) -> bool {
        self.width >= 120 && self.height >= 40
    }
}

/// Helper function to center a rectangle
pub fn center_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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

/// Create a margin around a rect
pub fn margin(horizontal: u16, vertical: u16) -> ratatui::layout::Margin {
    ratatui::layout::Margin { horizontal, vertical }
}