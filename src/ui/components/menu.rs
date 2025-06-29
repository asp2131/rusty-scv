use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};
use std::time::Duration;

use crate::ui::{
    animations::{AnimationState, EasingFunction},
    themes::Theme,
};

/// Menu item with animation support
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub title: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub enabled: bool,
    pub hotkey: Option<char>,
}

impl MenuItem {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: None,
            icon: None,
            enabled: true,
            hotkey: None,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn with_hotkey(mut self, key: char) -> Self {
        self.hotkey = Some(key);
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

/// Animated menu widget with smooth transitions and effects
pub struct AnimatedMenu {
    items: Vec<MenuItem>,
    selected: usize,
    title: Option<String>,
    show_help: bool,
    show_borders: bool,
    animation_offset: f32,
    highlight_animation: f32,
    entrance_animation: f32,
}

impl AnimatedMenu {
    pub fn new(items: Vec<MenuItem>) -> Self {
        Self {
            items,
            selected: 0,
            title: None,
            show_help: true,
            show_borders: true,
            animation_offset: 0.0,
            highlight_animation: 0.0,
            entrance_animation: 0.0,
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_help(mut self, show: bool) -> Self {
        self.show_help = show;
        self
    }

    pub fn with_borders(mut self, show: bool) -> Self {
        self.show_borders = show;
        self
    }

    pub fn select_next(&mut self) {
        if !self.items.is_empty() {
            self.selected = (self.selected + 1) % self.items.len();
            self.trigger_selection_animation();
        }
    }

    pub fn select_previous(&mut self) {
        if !self.items.is_empty() {
            self.selected = if self.selected == 0 {
                self.items.len() - 1
            } else {
                self.selected - 1
            };
            self.trigger_selection_animation();
        }
    }

    pub fn select_item(&mut self, index: usize) {
        if index < self.items.len() {
            self.selected = index;
            self.trigger_selection_animation();
        }
    }

    pub fn selected_item(&self) -> Option<&MenuItem> {
        self.items.get(self.selected)
    }

    pub fn selected_index(&self) -> usize {
        self.selected
    }

    pub fn items(&self) -> &[MenuItem] {
        &self.items
    }

    pub fn update(&mut self, delta_time: Duration, animation_state: &AnimationState) {
        // Update entrance animation
        if self.entrance_animation < 1.0 {
            self.entrance_animation += delta_time.as_secs_f32() * 3.0; // 3x speed
            self.entrance_animation = self.entrance_animation.min(1.0);
        }

        // Update highlight animation (oscillating)
        self.highlight_animation += delta_time.as_secs_f32() * 2.0;
        
        // Use menu highlight from animation state if available
        if let Some(target) = animation_state.menu_highlight.value().checked_sub(self.selected as u16) {
            self.animation_offset = target as f32 * 0.1; // Subtle offset effect
        }
    }

    fn trigger_selection_animation(&mut self) {
        // Reset highlight animation for new selection
        self.highlight_animation = 0.0;
    }

    pub fn trigger_entrance(&mut self) {
        self.entrance_animation = 0.0;
    }
}

impl Widget for &mut AnimatedMenu {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Apply entrance animation
        let entrance_progress = ease_out_cubic(self.entrance_animation);
        
        // Create the main block
        let block = if self.show_borders {
            let mut block = Block::default().borders(Borders::ALL);
            if let Some(ref title) = self.title {
                block = block.title(title.as_str());
            }
            block
        } else {
            Block::default()
        };

        let inner_area = if self.show_borders {
            block.inner(area)
        } else {
            area
        };

        // Render the block first
        if self.show_borders {
            block.render(area, buf);
        }

        // Calculate item areas
        let help_height = if self.show_help { 2 } else { 0 };
        let available_height = inner_area.height.saturating_sub(help_height);
        
        let menu_area = Rect {
            x: inner_area.x,
            y: inner_area.y,
            width: inner_area.width,
            height: available_height,
        };

        // Render menu items with animations
        self.render_menu_items(menu_area, buf, entrance_progress);

        // Render help text
        if self.show_help {
            let help_area = Rect {
                x: inner_area.x,
                y: inner_area.y + available_height,
                width: inner_area.width,
                height: help_height,
            };
            self.render_help(help_area, buf);
        }
    }
}


impl AnimatedMenu {
    fn render_menu_items(&mut self, area: Rect, buf: &mut Buffer, entrance_progress: f32) {
        let theme = &crate::ui::themes::THEMES.neon_night; // TODO: Get from context
        
        for (i, item) in self.items.iter().enumerate() {
            if i as u16 >= area.height {
                break; // Don't render items that won't fit
            }

            let item_y = area.y + i as u16;
            let is_selected = i == self.selected;
            
            // Calculate animation offsets
            let item_entrance_delay = i as f32 * 0.1; // Stagger entrance animations
            let item_entrance_progress = ((entrance_progress - item_entrance_delay) * 2.0).clamp(0.0, 1.0);
            
            // Slide in from the left
            let slide_offset = ((1.0 - item_entrance_progress) * 10.0) as u16;
            let item_x = area.x + slide_offset;
            let item_width = area.width.saturating_sub(slide_offset);
            
            if item_width == 0 {
                continue; // Skip if no width available
            }

            let item_area = Rect {
                x: item_x,
                y: item_y,
                width: item_width,
                height: 1,
            };

            // Calculate selection highlight with pulse animation
            let mut style = if is_selected {
                let pulse = (self.highlight_animation.sin() * 0.3 + 0.7).clamp(0.4, 1.0);
                let highlight_color = interpolate_color(theme.highlight, theme.primary, pulse);
                Style::default()
                    .fg(theme.text)
                    .bg(highlight_color)
                    .add_modifier(Modifier::BOLD)
            } else if item.enabled {
                Style::default().fg(theme.text)
            } else {
                Style::default().fg(theme.text_secondary)
            };

            // Build the item text
            let mut spans = Vec::new();
            
            // Selection indicator
            if is_selected {
                spans.push(Span::styled("▶ ", theme.primary_text()));
            } else {
                spans.push(Span::raw("  "));
            }

            // Icon
            if let Some(ref icon) = item.icon {
                spans.push(Span::styled(format!("{} ", icon), style));
            }

            // Title
            spans.push(Span::styled(&item.title, style));

            // Hotkey
            if let Some(hotkey) = item.hotkey {
                spans.push(Span::raw(" "));
                spans.push(Span::styled(
                    format!("({})", hotkey),
                    Style::default().fg(theme.text_secondary),
                ));
            }

            // Render the line
            let line = Line::from(spans);
            let paragraph = Paragraph::new(line);
            paragraph.render(item_area, buf);

            // Render description on next line if selected and available
            if is_selected && item.description.is_some() && item_y + 1 < area.y + area.height {
                let desc_area = Rect {
                    x: area.x + 4, // Indent description
                    y: item_y + 1,
                    width: area.width.saturating_sub(4),
                    height: 1,
                };

                let desc_text = item.description.as_ref().unwrap();
                let desc_line = Line::from(Span::styled(
                    desc_text,
                    Style::default().fg(theme.text_secondary).add_modifier(Modifier::ITALIC),
                ));
                let desc_paragraph = Paragraph::new(desc_line);
                desc_paragraph.render(desc_area, buf);
            }
        }
    }

    fn render_help(&self, area: Rect, buf: &mut Buffer) {
        let theme = &crate::ui::themes::THEMES.neon_night;
        
        let help_text = vec![
            Line::from(vec![
                Span::styled("↑/k", theme.primary_text()),
                Span::raw(" up • "),
                Span::styled("↓/j", theme.primary_text()),
                Span::raw(" down • "),
                Span::styled("Enter", theme.primary_text()),
                Span::raw(" select • "),
                Span::styled("q", theme.primary_text()),
                Span::raw(" quit"),
            ]),
        ];

        let help_paragraph = Paragraph::new(help_text)
            .style(Style::default().fg(theme.text_secondary))
            .alignment(Alignment::Center);
        
        help_paragraph.render(area, buf);
    }
}

// Animation easing functions
fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

// Color interpolation helper
fn interpolate_color(start: Color, end: Color, t: f32) -> Color {
    match (start, end) {
        (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => {
            let r = (r1 as f32 + (r2 as f32 - r1 as f32) * t) as u8;
            let g = (g1 as f32 + (g2 as f32 - g1 as f32) * t) as u8;
            let b = (b1 as f32 + (b2 as f32 - b1 as f32) * t) as u8;
            Color::Rgb(r, g, b)
        },
        _ => if t < 0.5 { start } else { end },
    }
}

/// Builder for creating animated menus easily
pub struct MenuBuilder {
    items: Vec<MenuItem>,
    title: Option<String>,
    show_help: bool,
    show_borders: bool,
}

impl MenuBuilder {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            title: None,
            show_help: true,
            show_borders: true,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn simple_item(mut self, title: impl Into<String>) -> Self {
        self.items.push(MenuItem::new(title));
        self
    }

    pub fn item_with_desc(mut self, title: impl Into<String>, desc: impl Into<String>) -> Self {
        self.items.push(MenuItem::new(title).with_description(desc));
        self
    }

    pub fn separator(mut self) -> Self {
        self.items.push(MenuItem::new("─".repeat(20)).disabled());
        self
    }

    pub fn show_help(mut self, show: bool) -> Self {
        self.show_help = show;
        self
    }

    pub fn show_borders(mut self, show: bool) -> Self {
        self.show_borders = show;
        self
    }

    pub fn build(self) -> AnimatedMenu {
        AnimatedMenu::new(self.items)
            .with_help(self.show_help)
            .with_borders(self.show_borders)
            .apply_title(self.title)
    }
}

impl AnimatedMenu {
    fn apply_title(mut self, title: Option<String>) -> Self {
        if let Some(title) = title {
            self.title = Some(title);
        }
        self
    }
}

impl Default for MenuBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Predefined menu configurations for common use cases
pub struct MenuPresets;

impl MenuPresets {
    /// Create a main menu with standard options
    pub fn main_menu() -> AnimatedMenu {
        MenuBuilder::new()
            .title("🎓 Student Code Viewer")
            .item(MenuItem::new("Manage Classes")
                .with_description("Select and manage an existing class")
                .with_icon("📚")
                .with_hotkey('m'))
            .item(MenuItem::new("Create Class")
                .with_description("Create a new class")
                .with_icon("➕")
                .with_hotkey('c'))
            .separator()
            .item(MenuItem::new("Settings")
                .with_description("Configure application settings")
                .with_icon("⚙️")
                .with_hotkey('s'))
            .item(MenuItem::new("Quit")
                .with_description("Exit the application")
                .with_icon("🚪")
                .with_hotkey('q'))
            .build()
    }

    /// Create a class management menu
    pub fn class_management(class_name: &str) -> AnimatedMenu {
        MenuBuilder::new()
            .title(format!("📚 Managing: {}", class_name))
            .item(MenuItem::new("Manage Students")
                .with_description("Add or remove students")
                .with_icon("👥")
                .with_hotkey('s'))
            .item(MenuItem::new("Manage Repositories")
                .with_description("Clone, pull, or clean repositories")
                .with_icon("📁")
                .with_hotkey('r'))
            .item(MenuItem::new("View GitHub Activity")
                .with_description("Check student GitHub activity")
                .with_icon("📊")
                .with_hotkey('a'))
            .separator()
            .item(MenuItem::new("Delete Class")
                .with_description("Delete this class and its data")
                .with_icon("🗑️")
                .with_hotkey('d'))
            .item(MenuItem::new("Back")
                .with_description("Return to main menu")
                .with_icon("↩️")
                .with_hotkey('b'))
            .build()
    }

    /// Create a student management menu
    pub fn student_management(class_name: &str) -> AnimatedMenu {
        MenuBuilder::new()
            .title(format!("👥 Students: {}", class_name))
            .item(MenuItem::new("Add Students")
                .with_description("Add new students to this class")
                .with_icon("➕")
                .with_hotkey('a'))
            .item(MenuItem::new("Remove Student")
                .with_description("Remove a student from this class")
                .with_icon("➖")
                .with_hotkey('r'))
            .item(MenuItem::new("View Student List")
                .with_description("View all students in this class")
                .with_icon("📋")
                .with_hotkey('v'))
            .separator()
            .item(MenuItem::new("Back")
                .with_description("Return to class management")
                .with_icon("↩️")
                .with_hotkey('b'))
            .build()
    }

    /// Create a repository management menu
    pub fn repository_management(class_name: &str) -> AnimatedMenu {
        MenuBuilder::new()
            .title(format!("📁 Repositories: {}", class_name))
            .item(MenuItem::new("Clone All Repos")
                .with_description("Clone all student repositories")
                .with_icon("⬇️")
                .with_hotkey('c'))
            .item(MenuItem::new("Pull All Repos")
                .with_description("Pull updates for all repositories")
                .with_icon("🔄")
                .with_hotkey('p'))
            .item(MenuItem::new("Clean All Repos")
                .with_description("Remove all cloned repositories")
                .with_icon("🧹")
                .with_hotkey('x'))
            .separator()
            .item(MenuItem::new("Back")
                .with_description("Return to class management")
                .with_icon("↩️")
                .with_hotkey('b'))
            .build()
    }

    /// Create a GitHub activity menu
    pub fn github_activity(class_name: &str) -> AnimatedMenu {
        MenuBuilder::new()
            .title(format!("📊 GitHub Activity: {}", class_name))
            .item(MenuItem::new("Week View")
                .with_description("View activity for the past week")
                .with_icon("📅")
                .with_hotkey('w'))
            .item(MenuItem::new("Latest Activity")
                .with_description("Check latest commit times")
                .with_icon("🕒")
                .with_hotkey('l'))
            .item(MenuItem::new("Activity Heatmap")
                .with_description("View contribution heatmap")
                .with_icon("🔥")
                .with_hotkey('h'))
            .separator()
            .item(MenuItem::new("Back")
                .with_description("Return to class management")
                .with_icon("↩️")
                .with_hotkey('b'))
            .build()
    }
}