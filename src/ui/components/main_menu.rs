use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
    Frame, backend::Backend,
};
use std::{time::Duration, future::Future, pin::Pin};

use crate::{
    app::{AppEvent, AppState},
    ui::{
        animations::AnimationState,
        components::menu::{AnimatedMenu, MenuPresets},
        screens::{Screen, ScreenType},
        themes::{Theme, AsciiArt},
    },
};

pub struct MainMenuScreen {
    menu: AnimatedMenu,
    logo_animation_time: f32,
    background_particles: Vec<BackgroundParticle>,
    show_logo: bool,
}

impl MainMenuScreen {
    pub fn new() -> Self {
        let mut menu = MenuPresets::main_menu();
        menu.trigger_entrance();

        Self {
            menu,
            logo_animation_time: 0.0,
            background_particles: generate_background_particles(),
            show_logo: true,
        }
    }

    fn render_logo(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let logo_lines = AsciiArt::logo();
        let logo_height = logo_lines.len() as u16;
        
        if area.height < logo_height + 2 {
            return; // Not enough space
        }

        // Center the logo
        let logo_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: logo_height,
        };

        // Animate logo with color cycling
        let time_factor = self.logo_animation_time * 0.5;
        for (i, line) in logo_lines.iter().enumerate() {
            let line_y = logo_area.y + i as u16;
            if line_y >= area.y + area.height {
                break;
            }

            // Create a rainbow effect
            let hue_offset = (time_factor + i as f32 * 0.2).sin() * 0.5 + 0.5;
            let color = if hue_offset < 0.33 {
                theme.primary
            } else if hue_offset < 0.66 {
                theme.secondary
            } else {
                theme.accent
            };

            // Add a glow effect
            let glow_intensity = (self.logo_animation_time * 2.0 + i as f32 * 0.5).sin() * 0.3 + 0.7;
            let final_color = interpolate_color(color, theme.text, glow_intensity);

            let line_area = Rect {
                x: logo_area.x,
                y: line_y,
                width: logo_area.width,
                height: 1,
            };

            let styled_line = Line::from(Span::styled(
                *line,
                Style::default()
                    .fg(final_color)
                    .add_modifier(Modifier::BOLD),
            ));

            let paragraph = Paragraph::new(styled_line).alignment(Alignment::Center);
            paragraph.render(line_area, buf);
        }
    }

    fn render_background_particles(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        for particle in &self.background_particles {
            let x = (particle.x * area.width as f32) as u16;
            let y = (particle.y * area.height as f32) as u16;

            if x < area.width && y < area.height {
                let particle_area = Rect {
                    x: area.x + x,
                    y: area.y + y,
                    width: 1,
                    height: 1,
                };

                // Animate particle brightness
                let brightness = (self.logo_animation_time * particle.speed + particle.phase).sin() * 0.5 + 0.5;
                let alpha = (brightness * particle.alpha).clamp(0.0, 1.0);
                
                if alpha > 0.1 {
                    let color = interpolate_color(theme.background, particle.color, alpha);
                    
                    let particle_char = match particle.particle_type {
                        ParticleType::Dot => "·",
                        ParticleType::Star => "✦",
                        ParticleType::Plus => "+",
                        ParticleType::Diamond => "◆",
                    };

                    let particle_span = Span::styled(
                        particle_char,
                        Style::default().fg(color),
                    );

                    let particle_line = Line::from(particle_span);
                    let paragraph = Paragraph::new(particle_line);
                    paragraph.render(particle_area, buf);
                }
            }
        }
    }

    fn render_footer(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        if area.height < 3 {
            return;
        }

        let footer_area = Rect {
            x: area.x,
            y: area.y + area.height - 2,
            width: area.width,
            height: 2,
        };

        // Version and build info
        let version_text = format!("Student Code Viewer v{} • Built with ❤️ and Rust", env!("CARGO_PKG_VERSION"));
        let version_line = Line::from(vec![
            Span::styled(
                version_text,
                Style::default().fg(theme.text_secondary),
            ),
        ]);

        let version_paragraph = Paragraph::new(version_line).alignment(Alignment::Center);
        version_paragraph.render(footer_area, buf);

        // GitHub info
        let github_area = Rect {
            x: footer_area.x,
            y: footer_area.y + 1,
            width: footer_area.width,
            height: 1,
        };

        let github_line = Line::from(vec![
            Span::styled("🐙 ", Style::default().fg(theme.accent)),
            Span::styled(
                "github.com/asp2131/student-code-viewer-rust",
                Style::default().fg(theme.text_secondary),
            ),
        ]);

        let github_paragraph = Paragraph::new(github_line).alignment(Alignment::Center);
        github_paragraph.render(github_area, buf);
    }
}

impl Screen for MainMenuScreen {
    fn screen_type(&self) -> ScreenType {
        ScreenType::MainMenu
    }

    fn handle_key_event<'a>(
        &'a mut self, 
        key: KeyEvent, 
        _state: &'a AppState
    ) -> Pin<Box<dyn Future<Output = Result<Option<AppEvent>>> + Send + 'a>> {
        let result = match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                // Show confirmation dialog or exit immediately
                Ok(Some(AppEvent::Quit))
            },
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Force exit without confirmation
                Ok(Some(AppEvent::Quit))
            },
            KeyCode::Char('r') => {
                // Reset menu animation
                self.menu.trigger_entrance();
                self.logo_animation_time = 0.0;
                Ok(None)
            },
            KeyCode::Char('l') => {
                // Toggle logo visibility
                self.show_logo = !self.show_logo;
                Ok(None)
            },
            KeyCode::Up | KeyCode::Char('k') => {
                self.menu.select_previous();
                Ok(None)
            },
            KeyCode::Down | KeyCode::Char('j') => {
                self.menu.select_next();
                Ok(None)
            },
            KeyCode::Enter | KeyCode::Char(' ') => {
                if let Some(selected_item) = self.menu.selected_item() {
                    match selected_item.title.as_str() {
                        "Create Class" => Ok(Some(AppEvent::NavigateToScreen(ScreenType::new(ScreenTypeVariant::CreateClass)))),
                        "Manage Classes" => Ok(Some(AppEvent::NavigateToScreen(ScreenType::new(ScreenTypeVariant::ClassSelection)))),
                        "Settings" => Ok(Some(AppEvent::NavigateToScreen(ScreenType::new(ScreenTypeVariant::Settings)))),
                        "Quit" => Ok(Some(AppEvent::Quit)),
                        _ => Ok(None),
                    }
                } else {
                    Ok(None)
                }
            },
            _ => Ok(None),
        };
        Box::pin(async { result })
    }

    fn update<'a>(
        &'a mut self,
        delta_time: Duration,
        state: &'a mut AppState,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        // Update animations
        let animation_state = AnimationState::default();
        self.menu.update(delta_time, &animation_state);
        self.logo_animation_time += delta_time.as_secs_f32();

        // Update background particles
        for particle in &mut self.background_particles {
            particle.update(delta_time);
        }

        Box::pin(async { Ok(()) })
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn render(
        &mut self,
        frame: &mut Frame<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
        area: Rect,
        _state: &AppState,
        animation_state: &AnimationState,
        theme: &Theme,
    ) {
        // Draw background particles
        // Background particles rendering simplified for compatibility
        // self.render_background_particles(area, frame.buffer_mut(), theme);
        
        // Create a centered area for the main content
        let popup_area = crate::ui::layout::center_rect(60, 80, area);
        
        // Create a block for the content
        let block = Block::default()
            .borders(Borders::NONE)
            .style(Style::default().bg(theme.background).fg(theme.text));
            
        frame.render_widget(block, popup_area);
        
        // Create inner area for content
        let inner_area = popup_area.inner(&crate::ui::layout::margin(1, 1));
        
        // Draw logo if enabled
        if self.show_logo {
            // Logo rendering simplified for compatibility
            // self.render_logo(inner_area, frame.buffer_mut(), theme);
        }
        
        // Draw menu
        let menu_area = if self.show_logo {
            // Position menu below logo
            Rect {
                x: inner_area.x,
                y: inner_area.y + 10, // Adjust based on logo height
                width: inner_area.width,
                height: inner_area.height.saturating_sub(10),
            }
        } else {
            // Center menu if no logo
            crate::ui::layout::center_rect(40, 20, inner_area)
        };
        
        // Render menu with title
        let menu_block = Block::default()
            .borders(Borders::NONE)
            .title_alignment(Alignment::Center);
            
        let menu_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(3),    // Menu items
            ])
            .split(menu_area);
            
        // Render title
        let title = Paragraph::new("Student Class Viewer")
            .alignment(Alignment::Center)
            .style(Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(theme.primary));
                
        frame.render_widget(title, menu_chunks[0]);
        
        // Render menu
        frame.render_widget(&mut self.menu, menu_chunks[1]);
        
        // Footer and celebration rendering simplified for compatibility
        // self.render_footer(area, buf, theme);
        // if let Some(celebration) = &animation_state.success_celebration {
        //     self.render_celebration_particles(area, buf, celebration, theme);
        // }
    }
}

impl MainMenuScreen {
    fn render_celebration_particles(&self, area: Rect, buf: &mut Buffer, celebration: &crate::ui::animations::CelebrationAnimation, theme: &Theme) {
        for particle in celebration.particles() {
            if !particle.is_alive() {
                continue;
            }

            let x = particle.x as u16;
            let y = particle.y as u16;

            if x < area.width && y < area.height {
                let particle_area = Rect {
                    x: area.x + x,
                    y: area.y + y,
                    width: 1,
                    height: 1,
                };

                let alpha = particle.alpha();
                let color = interpolate_color(theme.background, particle.color, alpha);

                let particle_span = Span::styled(
                    particle.character.to_string(),
                    Style::default().fg(color),
                );

                let particle_line = Line::from(particle_span);
                let paragraph = Paragraph::new(particle_line);
                paragraph.render(particle_area, buf);
            }
        }
    }
}

// Background particle system
#[derive(Debug, Clone)]
struct BackgroundParticle {
    x: f32,
    y: f32,
    speed: f32,
    phase: f32,
    alpha: f32,
    color: Color,
    particle_type: ParticleType,
}

#[derive(Debug, Clone)]
enum ParticleType {
    Dot,
    Star,
    Plus,
    Diamond,
}

impl BackgroundParticle {
    fn new(x: f32, y: f32) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        Self {
            x,
            y,
            speed: rng.gen_range(0.5..2.0),
            phase: rng.gen_range(0.0..std::f32::consts::TAU),
            alpha: rng.gen_range(0.1..0.5),
            color: match rng.gen_range(0..4) {
                0 => Color::Rgb(0, 212, 255),   // Electric blue
                1 => Color::Rgb(255, 27, 141),  // Hot pink
                2 => Color::Rgb(0, 255, 148),   // Neon green
                _ => Color::Rgb(255, 184, 0),   // Amber
            },
            particle_type: match rng.gen_range(0..4) {
                0 => ParticleType::Dot,
                1 => ParticleType::Star,
                2 => ParticleType::Plus,
                _ => ParticleType::Diamond,
            },
        }
    }

    fn update(&mut self, _delta_time: Duration) {
        // Particles are mostly static with animated brightness
        // Could add slow drifting motion here if desired
    }
}

fn generate_background_particles() -> Vec<BackgroundParticle> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut particles = Vec::new();

    // Generate sparse background particles
    for _ in 0..30 {
        let x = rng.gen_range(0.0..1.0);
        let y = rng.gen_range(0.0..1.0);
        particles.push(BackgroundParticle::new(x, y));
    }

    particles
}

// Helper function for color interpolation
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