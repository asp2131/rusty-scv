use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use crossterm::event::{KeyCode, KeyEvent};
use crate::ui::themes::Theme;

/// A reusable confirmation dialog component
pub struct ConfirmationDialog {
    title: String,
    message: String,
    yes_text: String,
    no_text: String,
    is_visible: bool,
}

impl ConfirmationDialog {
    /// Create a new confirmation dialog
    pub fn new(title: &str, message: &str) -> Self {
        Self {
            title: title.to_string(),
            message: message.to_string(),
            yes_text: "Yes".to_string(),
            no_text: "No".to_string(),
            is_visible: false,
        }
    }

    /// Set custom text for the "yes" button
    pub fn with_yes_text(mut self, text: &str) -> Self {
        self.yes_text = text.to_string();
        self
    }

    /// Set custom text for the "no" button
    pub fn with_no_text(mut self, text: &str) -> Self {
        self.no_text = text.to_string();
        self
    }

    /// Show the dialog
    pub fn show(&mut self) {
        self.is_visible = true;
    }

    /// Hide the dialog
    pub fn hide(&mut self) {
        self.is_visible = false;
    }

    /// Check if the dialog is currently visible
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    /// Handle key events for the dialog
    /// Returns true if "yes" was selected, false if "no" was selected or dialog was cancelled
    pub fn handle_key_event(&mut self, key: KeyEvent) -> Option<bool> {
        if !self.is_visible {
            return None;
        }

        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.hide();
                Some(true)
            },
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.hide();
                Some(false)
            },
            _ => None,
        }
    }

    /// Render the dialog
    pub fn render<B: Backend>(&self, frame: &mut Frame<B>, area: Rect, theme: &Theme) {
        if !self.is_visible {
            return;
        }

        // Create a centered popup
        let popup_area = centered_rect(50, 30, area);
        
        // Clear the background
        frame.render_widget(Clear, popup_area);
        
        // Create the block for the dialog
        let block = Block::default()
            .title(self.title.clone())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.warning))
            .title_style(Style::default().fg(theme.warning).add_modifier(Modifier::BOLD));
            
        let inner_area = block.inner(popup_area);
        frame.render_widget(block, popup_area);
        
        // Split the inner area for message and buttons
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),     // Message area
                Constraint::Length(3),  // Buttons
            ])
            .split(inner_area);
            
        // Render the message
        let message = Paragraph::new(self.message.clone())
            .alignment(Alignment::Center)
            .style(Style::default().fg(theme.text));
            
        frame.render_widget(message, chunks[0]);
        
        // Render the buttons
        let button_text = Line::from(vec![
            Span::styled("[", Style::default().fg(theme.text_secondary)),
            Span::styled(format!("Y - {}", self.yes_text), Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
            Span::styled("]", Style::default().fg(theme.text_secondary)),
            Span::styled("   ", Style::default().fg(theme.text_secondary)),
            Span::styled("[", Style::default().fg(theme.text_secondary)),
            Span::styled(format!("N - {}", self.no_text), Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled("]", Style::default().fg(theme.text_secondary)),
        ]);
        
        let buttons = Paragraph::new(button_text)
            .alignment(Alignment::Center);
            
        frame.render_widget(buttons, chunks[1]);
    }
}

/// Helper function to create a centered rectangle
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
