use ratatui::style::{Color, Style, Modifier};

/// Color theme for the application
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: &'static str,
    
    // Primary colors
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    
    // Status colors
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    
    // UI colors
    pub background: Color,
    pub surface: Color,
    pub text: Color,
    pub text_secondary: Color,
    pub border: Color,
    pub highlight: Color,
    pub selection: Color,
    
    // GitHub activity colors
    pub activity_none: Color,
    pub activity_low: Color,
    pub activity_medium: Color,
    pub activity_high: Color,
    pub activity_max: Color,
}

impl Theme {
    /// Get a style for primary text
    pub fn primary_text(&self) -> Style {
        Style::default().fg(self.primary).add_modifier(Modifier::BOLD)
    }

    /// Get a style for secondary text
    pub fn secondary_text(&self) -> Style {
        Style::default().fg(self.text_secondary)
    }

    /// Get a style for success messages
    pub fn success_text(&self) -> Style {
        Style::default().fg(self.success).add_modifier(Modifier::BOLD)
    }

    /// Get a style for error messages
    pub fn error_text(&self) -> Style {
        Style::default().fg(self.error).add_modifier(Modifier::BOLD)
    }

    /// Get a style for highlighted/selected items
    pub fn highlight_style(&self) -> Style {
        Style::default()
            .fg(self.text)
            .bg(self.highlight)
            .add_modifier(Modifier::BOLD)
    }

    /// Get a style for borders
    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }

    /// Get a style for focused borders
    pub fn border_focused_style(&self) -> Style {
        Style::default().fg(self.primary)
    }

    /// Get activity level color for GitHub activity
    pub fn activity_color(&self, level: ActivityLevel) -> Color {
        match level {
            ActivityLevel::None => self.activity_none,
            ActivityLevel::Low => self.activity_low,
            ActivityLevel::Medium => self.activity_medium,
            ActivityLevel::High => self.activity_high,
            ActivityLevel::Max => self.activity_max,
        }
    }
}

/// Activity levels for GitHub activity visualization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityLevel {
    None,
    Low,
    Medium,
    High,
    Max,
}

impl ActivityLevel {
    pub fn from_commit_count(count: u32) -> Self {
        match count {
            0 => Self::None,
            1..=2 => Self::Low,
            3..=5 => Self::Medium,
            6..=10 => Self::High,
            _ => Self::Max,
        }
    }
}

/// Collection of available themes
pub struct Themes {
    pub neon_night: Theme,
    pub cyberpunk: Theme,
    pub ocean_breeze: Theme,
    pub forest_dark: Theme,
    pub sunset_glow: Theme,
}

pub static THEMES: Themes = Themes {
    neon_night: Theme {
        name: "Neon Night",
        primary: Color::Rgb(0, 212, 255),        // Electric blue
        secondary: Color::Rgb(255, 27, 141),     // Hot pink
        accent: Color::Rgb(0, 255, 148),         // Neon green
        
        success: Color::Rgb(0, 255, 148),        // Neon green
        warning: Color::Rgb(255, 184, 0),        // Amber
        error: Color::Rgb(255, 107, 107),        // Coral red
        info: Color::Rgb(0, 212, 255),           // Electric blue
        
        background: Color::Rgb(10, 10, 10),      // Deep space
        surface: Color::Rgb(26, 26, 26),         // Dark gray
        text: Color::Rgb(255, 255, 255),         // White
        text_secondary: Color::Rgb(170, 170, 170), // Light gray
        border: Color::Rgb(68, 68, 68),          // Medium gray
        highlight: Color::Rgb(255, 27, 141),     // Hot pink
        selection: Color::Rgb(0, 212, 255),      // Electric blue
        
        activity_none: Color::Rgb(40, 40, 40),
        activity_low: Color::Rgb(0, 100, 255),
        activity_medium: Color::Rgb(0, 180, 255),
        activity_high: Color::Rgb(0, 255, 180),
        activity_max: Color::Rgb(0, 255, 80),
    },

    cyberpunk: Theme {
        name: "Cyberpunk",
        primary: Color::Rgb(255, 0, 255),        // Magenta
        secondary: Color::Rgb(0, 255, 255),      // Cyan
        accent: Color::Rgb(255, 255, 0),         // Yellow
        
        success: Color::Rgb(0, 255, 0),          // Lime green
        warning: Color::Rgb(255, 165, 0),        // Orange
        error: Color::Rgb(255, 0, 0),            // Red
        info: Color::Rgb(0, 255, 255),           // Cyan
        
        background: Color::Rgb(0, 0, 0),         // Black
        surface: Color::Rgb(20, 0, 20),          // Dark purple
        text: Color::Rgb(0, 255, 0),             // Green
        text_secondary: Color::Rgb(128, 255, 128), // Light green
        border: Color::Rgb(255, 0, 255),         // Magenta
        highlight: Color::Rgb(255, 255, 0),      // Yellow
        selection: Color::Rgb(255, 0, 255),      // Magenta
        
        activity_none: Color::Rgb(50, 0, 50),
        activity_low: Color::Rgb(255, 0, 100),
        activity_medium: Color::Rgb(255, 0, 200),
        activity_high: Color::Rgb(255, 100, 255),
        activity_max: Color::Rgb(255, 200, 255),
    },

    ocean_breeze: Theme {
        name: "Ocean Breeze",
        primary: Color::Rgb(52, 152, 219),       // Blue
        secondary: Color::Rgb(26, 188, 156),     // Turquoise
        accent: Color::Rgb(46, 204, 113),        // Green
        
        success: Color::Rgb(46, 204, 113),       // Green
        warning: Color::Rgb(241, 196, 15),       // Yellow
        error: Color::Rgb(231, 76, 60),          // Red
        info: Color::Rgb(52, 152, 219),          // Blue
        
        background: Color::Rgb(12, 20, 31),      // Dark blue
        surface: Color::Rgb(23, 32, 42),         // Navy
        text: Color::Rgb(236, 240, 241),         // Light blue-white
        text_secondary: Color::Rgb(149, 165, 166), // Gray-blue
        border: Color::Rgb(52, 73, 94),          // Dark gray
        highlight: Color::Rgb(26, 188, 156),     // Turquoise
        selection: Color::Rgb(52, 152, 219),     // Blue
        
        activity_none: Color::Rgb(30, 40, 50),
        activity_low: Color::Rgb(52, 152, 219),
        activity_medium: Color::Rgb(26, 188, 156),
        activity_high: Color::Rgb(46, 204, 113),
        activity_max: Color::Rgb(155, 227, 152),
    },

    forest_dark: Theme {
        name: "Forest Dark",
        primary: Color::Rgb(76, 175, 80),        // Green
        secondary: Color::Rgb(139, 195, 74),     // Light green
        accent: Color::Rgb(255, 235, 59),        // Yellow
        
        success: Color::Rgb(76, 175, 80),        // Green
        warning: Color::Rgb(255, 193, 7),        // Amber
        error: Color::Rgb(244, 67, 54),          // Red
        info: Color::Rgb(33, 150, 243),          // Blue
        
        background: Color::Rgb(18, 32, 18),      // Dark green
        surface: Color::Rgb(28, 42, 28),         // Forest green
        text: Color::Rgb(232, 245, 233),         // Light green-white
        text_secondary: Color::Rgb(165, 214, 167), // Light green
        border: Color::Rgb(56, 87, 35),          // Dark green
        highlight: Color::Rgb(139, 195, 74),     // Light green
        selection: Color::Rgb(76, 175, 80),      // Green
        
        activity_none: Color::Rgb(40, 50, 40),
        activity_low: Color::Rgb(76, 175, 80),
        activity_medium: Color::Rgb(139, 195, 74),
        activity_high: Color::Rgb(174, 213, 129),
        activity_max: Color::Rgb(220, 237, 200),
    },

    sunset_glow: Theme {
        name: "Sunset Glow",
        primary: Color::Rgb(255, 87, 34),        // Orange
        secondary: Color::Rgb(255, 152, 0),      // Amber
        accent: Color::Rgb(255, 193, 7),         // Yellow
        
        success: Color::Rgb(139, 195, 74),       // Light green
        warning: Color::Rgb(255, 193, 7),        // Yellow
        error: Color::Rgb(244, 67, 54),          // Red
        info: Color::Rgb(103, 58, 183),          // Purple
        
        background: Color::Rgb(33, 17, 8),       // Dark brown
        surface: Color::Rgb(51, 25, 12),         // Brown
        text: Color::Rgb(255, 245, 238),         // Warm white
        text_secondary: Color::Rgb(188, 170, 164), // Warm gray
        border: Color::Rgb(121, 85, 72),         // Brown
        highlight: Color::Rgb(255, 152, 0),      // Amber
        selection: Color::Rgb(255, 87, 34),      // Orange
        
        activity_none: Color::Rgb(60, 40, 30),
        activity_low: Color::Rgb(255, 87, 34),
        activity_medium: Color::Rgb(255, 152, 0),
        activity_high: Color::Rgb(255, 193, 7),
        activity_max: Color::Rgb(255, 235, 59),
    },
};

/// Utility functions for working with themes
impl Themes {
    pub fn get_theme_by_name(&self, name: &str) -> Option<&Theme> {
        match name {
            "neon_night" => Some(&self.neon_night),
            "cyberpunk" => Some(&self.cyberpunk),
            "ocean_breeze" => Some(&self.ocean_breeze),
            "forest_dark" => Some(&self.forest_dark),
            "sunset_glow" => Some(&self.sunset_glow),
            _ => None,
        }
    }

    pub fn list_theme_names(&self) -> Vec<&'static str> {
        vec![
            "neon_night",
            "cyberpunk", 
            "ocean_breeze",
            "forest_dark",
            "sunset_glow",
        ]
    }

    pub fn default_theme(&self) -> &Theme {
        &self.neon_night
    }
}

/// Gradient utility for creating smooth color transitions
pub struct ColorGradient {
    start: Color,
    end: Color,
    steps: usize,
}

impl ColorGradient {
    pub fn new(start: Color, end: Color, steps: usize) -> Self {
        Self { start, end, steps }
    }

    pub fn color_at(&self, step: usize) -> Color {
        if step >= self.steps {
            return self.end;
        }

        let t = step as f32 / (self.steps - 1) as f32;
        interpolate_color(self.start, self.end, t)
    }

    pub fn colors(&self) -> Vec<Color> {
        (0..self.steps)
            .map(|i| self.color_at(i))
            .collect()
    }
}

/// Interpolate between two colors
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

/// ASCII art and decorative elements
pub struct AsciiArt;

impl AsciiArt {
    pub fn logo() -> Vec<&'static str> {
        vec![
            "  ███████╗ ██████╗██╗   ██╗",
            "  ██╔════╝██╔════╝██║   ██║",
            "  ███████╗██║     ██║   ██║",
            "  ╚════██║██║     ╚██╗ ██╔╝",
            "  ███████║╚██████╗ ╚████╔╝ ",
            "  ╚══════╝ ╚═════╝  ╚═══╝  ",
            "",
            "  Student Code Viewer",
        ]
    }

    pub fn github_octocat() -> Vec<&'static str> {
        vec![
            "        .-\"\"\"\"\"-.",
            "       /         \\",
            "      |  O     O  |",
            "      |     ∩     |",
            "       \\    ___   /",
            "        '-.......-'",
            "         |  ___  |",
            "         |_|   |_|",
        ]
    }

    pub fn loading_spinner_frames() -> Vec<&'static str> {
        vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]
    }

    pub fn celebration_confetti() -> Vec<&'static str> {
        vec!["🎉", "✨", "🎊", "⭐", "💫", "🌟", "✨"]
    }
}