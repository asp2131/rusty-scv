use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Gauge, Paragraph, Widget, Wrap},
};
use std::time::Duration;

use crate::ui::{
    animations::{AnimationState, SpinnerAnimation, ProgressAnimation},
    themes::Theme,
};

/// Different types of loading animations
#[derive(Debug, Clone)]
pub enum LoadingType {
    Spinner,
    ProgressBar { current: f32, total: f32 },
    Dots,
    Pulse,
    Matrix,
    Custom { frames: Vec<String> },
}

/// Animated loading widget with various visual effects
pub struct LoadingWidget {
    message: String,
    loading_type: LoadingType,
    spinner: SpinnerAnimation,
    progress: ProgressAnimation,
    pulse_animation: f32,
    matrix_chars: Vec<char>,
    show_percentage: bool,
    theme: Theme,
}

impl LoadingWidget {
    pub fn new(message: &str, animation_state: &AnimationState, theme: &Theme) -> Self {
        Self {
            message: message.to_string(),
            loading_type: LoadingType::Spinner,
            spinner: SpinnerAnimation::dots(),
            progress: ProgressAnimation::new(),
            pulse_animation: 0.0,
            matrix_chars: "ｦｧｨｩｪｫｬｭｮｯｰｱｲｳｴｵｶｷｸｹｺｻｼｽｾｿﾀﾁﾂﾃﾄﾅﾆﾇﾈﾉﾊﾋﾌﾍﾎﾏﾐﾑﾒﾓﾔﾕﾖﾗﾘﾙﾚﾛﾜﾝ"
                .chars()
                .collect(),
            show_percentage: false,
            theme: theme.clone(),
        }
    }

    pub fn with_spinner(mut self) -> Self {
        self.loading_type = LoadingType::Spinner;
        self
    }

    pub fn with_progress(mut self, current: f32, total: f32) -> Self {
        self.loading_type = LoadingType::ProgressBar { current, total };
        self.progress.set_progress(current / total);
        self.show_percentage = true;
        self
    }

    pub fn with_dots(mut self) -> Self {
        self.loading_type = LoadingType::Dots;
        self
    }

    pub fn with_pulse(mut self) -> Self {
        self.loading_type = LoadingType::Pulse;
        self
    }

    pub fn with_matrix(mut self) -> Self {
        self.loading_type = LoadingType::Matrix;
        self
    }

    pub fn update_progress(&mut self, current: f32, total: f32) {
        if let LoadingType::ProgressBar { .. } = self.loading_type {
            self.loading_type = LoadingType::ProgressBar { current, total };
            self.progress.set_progress(current / total);
        }
    }

    pub fn update(&mut self, delta_time: Duration) {
        self.spinner.update(delta_time);
        self.progress.update(delta_time);
        self.pulse_animation += delta_time.as_secs_f32() * 2.0; // 2 pulses per second
    }
}

impl Widget for LoadingWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create the loading container
        let block = Block::default()
            .title("Loading")
            .borders(Borders::ALL)
            .border_style(self.theme.border_focused_style())
            .title_style(self.theme.primary_text());

        let inner_area = block.inner(area);
        block.render(area, buf);

        // Layout the loading content
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Animation area
                Constraint::Length(2), // Message area
                Constraint::Min(0),    // Remaining space
            ])
            .split(inner_area);

        // Create a mutable copy of self for rendering
        let mut this = self;
        
        // Render the animation
        this.render_animation(chunks[0], buf);

        // Render the message
        this.render_message(chunks[1], buf);
    }
}

impl LoadingWidget {
    fn render_animation(&mut self, area: Rect, buf: &mut Buffer) {
        match &self.loading_type {
            LoadingType::Spinner => self.render_spinner(area, buf),
            LoadingType::ProgressBar { current, total } => {
                self.render_progress_bar(area, buf, *current, *total)
            },
            LoadingType::Dots => self.render_dots(area, buf),
            LoadingType::Pulse => self.render_pulse(area, buf),
            LoadingType::Matrix => self.render_matrix(area, buf),
            LoadingType::Custom { frames } => {
                let frames = frames.clone();
                self.render_custom(area, buf, &frames)
            },
        }
    }

    fn render_spinner(&mut self, area: Rect, buf: &mut Buffer) {
        let spinner_frame = self.spinner.current_frame();
        
        // Use static color for spinner
        let color = self.theme.primary;
        
        let spinner_text = format!("  {}  ", spinner_frame);
        let line = Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(spinner_frame, Style::default().fg(color).add_modifier(Modifier::BOLD)),
            Span::styled("  ", Style::default()),
        ]);

        let paragraph = Paragraph::new(line).alignment(Alignment::Center);
        paragraph.render(area, buf);

        // Add surrounding effect
        if area.height > 1 {
            let effect_area = Rect {
                x: area.x,
                y: area.y + 1,
                width: area.width,
                height: 1,
            };

            let effect_intensity = (self.pulse_animation.sin() * 5.0) as usize;
            let effect_char = match effect_intensity {
                0 => "·",
                1 => "⋅",
                2 => "•",
                3 => "●",
                _ => "◉",
            };

            let effect_line = Line::from(Span::styled(
                format!("{}  {}  {}", effect_char, effect_char, effect_char),
                Style::default().fg(self.theme.accent),
            ));

            let effect_paragraph = Paragraph::new(effect_line).alignment(Alignment::Center);
            effect_paragraph.render(effect_area, buf);
        }
    }

    fn render_progress_bar(&mut self, area: Rect, buf: &mut Buffer, current: f32, total: f32) {
        let progress = (current / total).clamp(0.0, 1.0);
        let percentage = (progress * 100.0) as u16;

        // Create static progress bar
        let _pulse = *self.progress.pulse.value();
        let bar_color = self.theme.primary;

        let gauge = Gauge::default()
            .block(Block::default())
            .gauge_style(Style::default().fg(bar_color).add_modifier(Modifier::BOLD))
            .percent(percentage)
            .label(if self.show_percentage {
                format!("{}%", percentage)
            } else {
                String::new()
            });

        gauge.render(area, buf);

        // Add animated progress indicators
        if area.height > 1 {
            let indicator_area = Rect {
                x: area.x,
                y: area.y + 1,
                width: area.width,
                height: 1,
            };

            let filled_width = (area.width as f32 * progress) as u16;
            let mut indicator_text = String::new();

            for i in 0..area.width {
                if i < filled_width {
                    let wave_offset = (self.pulse_animation + i as f32 * 0.3).sin();
                    let char = if wave_offset > 0.5 { "▲" } else { "△" };
                    indicator_text.push_str(char);
                } else {
                    indicator_text.push(' ');
                }
            }

            let indicator_line = Line::from(Span::styled(
                indicator_text,
                Style::default().fg(self.theme.accent),
            ));

            let indicator_paragraph = Paragraph::new(indicator_line);
            indicator_paragraph.render(indicator_area, buf);
        }
    }

    fn render_dots(&mut self, area: Rect, buf: &mut Buffer) {
        let time = self.pulse_animation;
        let mut dots = String::new();

        // Create a wave of dots
        for i in 0..8 {
            let wave = (time + i as f32 * 0.5).sin();
            let char = if wave > 0.7 {
                "●"
            } else if wave > 0.3 {
                "◐"
            } else if wave > -0.3 {
                "○"
            } else {
                " "
            };
            dots.push_str(char);
            if i < 7 {
                dots.push(' ');
            }
        }

        let line = Line::from(Span::styled(
            dots,
            Style::default().fg(self.theme.primary).add_modifier(Modifier::BOLD),
        ));

        let paragraph = Paragraph::new(line).alignment(Alignment::Center);
        paragraph.render(area, buf);
    }

    fn render_pulse(&mut self, area: Rect, buf: &mut Buffer) {
        let pulse = (self.pulse_animation.sin() * 0.5 + 0.5).clamp(0.0, 1.0);
        let intensity = (pulse * 3.0) as usize;
        
        let (char, color) = match intensity {
            0 => ("◇", self.theme.text_secondary),
            1 => ("◈", self.theme.primary),
            2 => ("◉", self.theme.accent),
            _ => ("⬢", self.theme.secondary),
        };

        let line = Line::from(Span::styled(
            format!("  {}  ", char),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ));

        let paragraph = Paragraph::new(line).alignment(Alignment::Center);
        paragraph.render(area, buf);

        // Add ripple effect
        if area.height > 1 {
            let ripple_intensity = ((pulse - 0.2).max(0.0) * 4.0) as usize;
            let ripple_char = match ripple_intensity {
                0 => " ",
                1 => "·",
                2 => "∘",
                _ => "○",
            };

            let ripple_area = Rect {
                x: area.x,
                y: area.y + 1,
                width: area.width,
                height: 1,
            };

            let ripple_line = Line::from(Span::styled(
                format!("{}   {}   {}", ripple_char, ripple_char, ripple_char),
                Style::default().fg(self.theme.text_secondary),
            ));

            let ripple_paragraph = Paragraph::new(ripple_line).alignment(Alignment::Center);
            ripple_paragraph.render(ripple_area, buf);
        }
    }

    fn render_matrix(&mut self, area: Rect, buf: &mut Buffer) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        // Generate matrix-style falling characters
        let mut matrix_text = String::new();
        let cols = (area.width / 2) as usize; // Each character takes 2 spaces

        for _ in 0..cols {
            if rng.gen_bool(0.3) { // 30% chance of character
                let char_idx = rng.gen_range(0..self.matrix_chars.len());
                let char = self.matrix_chars[char_idx];
                
                // Use static color for matrix
                let _intensity = rng.gen_range(0..4);
                let _color = self.theme.success;

                matrix_text.push(char);
            } else {
                matrix_text.push(' ');
            }
            matrix_text.push(' '); // Spacing
        }

        let line = Line::from(Span::styled(
            matrix_text,
            Style::default().fg(self.theme.success).add_modifier(Modifier::BOLD),
        ));

        let paragraph = Paragraph::new(line).alignment(Alignment::Center);
        paragraph.render(area, buf);
    }

    fn render_custom(&mut self, area: Rect, buf: &mut Buffer, frames: &[String]) {
        if frames.is_empty() {
            return;
        }

        let frame_index = ((self.pulse_animation * 2.0) as usize) % frames.len();
        let current_frame = &frames[frame_index];

        let line = Line::from(Span::styled(
            current_frame,
            Style::default().fg(self.theme.primary).add_modifier(Modifier::BOLD),
        ));

        let paragraph = Paragraph::new(line).alignment(Alignment::Center);
        paragraph.render(area, buf);
    }

    fn render_message(&mut self, area: Rect, buf: &mut Buffer) {
        let message_style = Style::default().fg(self.theme.text);
        let paragraph = Paragraph::new(self.message.as_str())
            .style(message_style)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        paragraph.render(area, buf);
    }
}


/// Preset loading widgets for common operations
pub struct LoadingPresets;

impl LoadingPresets {
    pub fn cloning_repos(theme: &Theme) -> LoadingWidget {
        LoadingWidget::new("Cloning repositories...", &Default::default(), theme)
            .with_spinner()
    }

    pub fn fetching_github_data(theme: &Theme) -> LoadingWidget {
        LoadingWidget::new("Fetching GitHub data...", &Default::default(), theme)
            .with_dots()
    }

    pub fn processing_students(current: f32, total: f32, theme: &Theme) -> LoadingWidget {
        LoadingWidget::new(
            &format!("Processing students ({}/{})", current as u32, total as u32),
            &Default::default(),
            theme,
        )
        .with_progress(current, total)
    }

    pub fn initializing(theme: &Theme) -> LoadingWidget {
        LoadingWidget::new("Initializing...", &Default::default(), theme)
            .with_pulse()
    }

    pub fn hacker_mode(theme: &Theme) -> LoadingWidget {
        LoadingWidget::new("Accessing the mainframe...", &Default::default(), theme)
            .with_matrix()
    }
}