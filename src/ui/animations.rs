use std::time::Duration;
use ratatui::style::{Color, Style};

/// Easing functions for smooth animations
#[derive(Debug, Clone, Copy)]
pub enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
    Elastic,
}

impl EasingFunction {
    pub fn apply(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            EasingFunction::Linear => t,
            EasingFunction::EaseIn => t * t,
            EasingFunction::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            EasingFunction::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - 2.0 * (1.0 - t) * (1.0 - t)
                }
            },
            EasingFunction::Bounce => {
                if t < 1.0 / 2.75 {
                    7.5625 * t * t
                } else if t < 2.0 / 2.75 {
                    let t = t - 1.5 / 2.75;
                    7.5625 * t * t + 0.75
                } else if t < 2.5 / 2.75 {
                    let t = t - 2.25 / 2.75;
                    7.5625 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / 2.75;
                    7.5625 * t * t + 0.984375
                }
            },
            EasingFunction::Elastic => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    let p = 0.3;
                    let s = p / 4.0;
                    -(2.0_f32.powf(10.0 * (t - 1.0)) * ((t - 1.0 - s) * (2.0 * std::f32::consts::PI) / p).sin())
                }
            },
        }
    }
}

/// Represents an animated value that can be interpolated over time
#[derive(Debug, Clone)]
pub struct AnimatedValue<T> {
    start_value: T,
    end_value: T,
    current_value: T,
    duration: Duration,
    elapsed: Duration,
    easing: EasingFunction,
    is_animating: bool,
}

impl<T> AnimatedValue<T> 
where 
    T: Clone + Interpolate,
{
    pub fn new(initial_value: T) -> Self {
        Self {
            start_value: initial_value.clone(),
            end_value: initial_value.clone(),
            current_value: initial_value,
            duration: Duration::from_millis(300),
            elapsed: Duration::ZERO,
            easing: EasingFunction::EaseInOut,
            is_animating: false,
        }
    }

    pub fn animate_to(&mut self, target: T, duration: Duration, easing: EasingFunction) {
        self.start_value = self.current_value.clone();
        self.end_value = target;
        self.duration = duration;
        self.easing = easing;
        self.elapsed = Duration::ZERO;
        self.is_animating = true;
    }

    pub fn update(&mut self, delta_time: Duration) {
        if !self.is_animating {
            return;
        }

        self.elapsed += delta_time;
        
        if self.elapsed >= self.duration {
            self.current_value = self.end_value.clone();
            self.is_animating = false;
        } else {
            let progress = self.elapsed.as_secs_f32() / self.duration.as_secs_f32();
            let eased_progress = self.easing.apply(progress);
            self.current_value = self.start_value.interpolate(&self.end_value, eased_progress);
        }
    }

    pub fn value(&self) -> &T {
        &self.current_value
    }

    pub fn is_animating(&self) -> bool {
        self.is_animating
    }

    pub fn set_immediate(&mut self, value: T) {
        self.start_value = value.clone();
        self.end_value = value.clone();
        self.current_value = value;
        self.is_animating = false;
    }
}

/// Trait for types that can be interpolated
pub trait Interpolate {
    fn interpolate(&self, other: &Self, t: f32) -> Self;
}

impl Interpolate for f32 {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        self + (other - self) * t
    }
}

impl Interpolate for u16 {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        (*self as f32 + (*other as f32 - *self as f32) * t) as u16
    }
}

impl Interpolate for Color {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        match (self, other) {
            (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => {
                let r = (*r1 as f32 + (*r2 as f32 - *r1 as f32) * t) as u8;
                let g = (*g1 as f32 + (*g2 as f32 - *g1 as f32) * t) as u8;
                let b = (*b1 as f32 + (*b2 as f32 - *b1 as f32) * t) as u8;
                Color::Rgb(r, g, b)
            },
            _ => if t < 0.5 { *self } else { *other },
        }
    }
}

/// Animation state for the entire application
#[derive(Debug)]
pub struct AnimationState {
    pub transition_progress: AnimatedValue<f32>,
    pub menu_highlight: AnimatedValue<u16>,
    pub loading_rotation: f32,
    pub particle_time: f32,
    pub background_pulse: AnimatedValue<f32>,
    pub success_celebration: Option<CelebrationAnimation>,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationState {
    pub fn new() -> Self {
        Self {
            transition_progress: AnimatedValue::new(0.0),
            menu_highlight: AnimatedValue::new(0),
            loading_rotation: 0.0,
            particle_time: 0.0,
            background_pulse: AnimatedValue::new(0.0),
            success_celebration: None,
        }
    }

    pub fn update(&mut self, delta_time: Duration) {
        self.transition_progress.update(delta_time);
        self.menu_highlight.update(delta_time);
        self.background_pulse.update(delta_time);
        
        // Update continuous animations
        let delta_secs = delta_time.as_secs_f32();
        self.loading_rotation += delta_secs * 360.0; // Full rotation per second
        self.particle_time += delta_secs;
        
        // Update celebration animation
        if let Some(ref mut celebration) = self.success_celebration {
            celebration.update(delta_time);
            if celebration.is_finished() {
                self.success_celebration = None;
            }
        }
    }

    pub fn trigger_transition(&mut self) {
        self.transition_progress.animate_to(
            1.0,
            Duration::from_millis(300),
            EasingFunction::EaseInOut,
        );
    }

    pub fn animate_menu_highlight(&mut self, target_index: u16) {
        self.menu_highlight.animate_to(
            target_index,
            Duration::from_millis(150),
            EasingFunction::EaseOut,
        );
    }

    pub fn trigger_success_celebration(&mut self) {
        self.success_celebration = Some(CelebrationAnimation::new());
    }

    pub fn pulse_background(&mut self) {
        self.background_pulse.animate_to(
            1.0,
            Duration::from_millis(200),
            EasingFunction::EaseInOut,
        );
        // Note: We'd need a callback system to animate back to 0.0
    }
}

/// Celebration animation for successful operations
#[derive(Debug)]
pub struct CelebrationAnimation {
    particles: Vec<Particle>,
    duration: Duration,
    elapsed: Duration,
}

impl CelebrationAnimation {
    pub fn new() -> Self {
        let mut particles = Vec::new();
        
        // Create confetti particles
        for _ in 0..50 {
            particles.push(Particle::new_confetti());
        }
        
        Self {
            particles,
            duration: Duration::from_secs(3),
            elapsed: Duration::ZERO,
        }
    }

    pub fn update(&mut self, delta_time: Duration) {
        self.elapsed += delta_time;
        
        for particle in &mut self.particles {
            particle.update(delta_time);
        }
    }

    pub fn is_finished(&self) -> bool {
        self.elapsed >= self.duration
    }

    pub fn particles(&self) -> &[Particle] {
        &self.particles
    }
}

/// Individual particle for effects
#[derive(Debug, Clone)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub color: Color,
    pub character: char,
    pub life: f32,
    pub max_life: f32,
}

impl Particle {
    pub fn new_confetti() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        Self {
            x: rng.gen_range(0.0..80.0),
            y: rng.gen_range(-10.0..0.0),
            velocity_x: rng.gen_range(-2.0..2.0),
            velocity_y: rng.gen_range(1.0..3.0),
            color: match rng.gen_range(0..5) {
                0 => Color::Red,
                1 => Color::Green,
                2 => Color::Blue,
                3 => Color::Yellow,
                4 => Color::Magenta,
                _ => Color::Cyan,
            },
            character: match rng.gen_range(0..4) {
                0 => '*',
                1 => '✨',
                2 => '🎉',
                _ => '●',
            },
            life: 3.0,
            max_life: 3.0,
        }
    }

    pub fn update(&mut self, delta_time: Duration) {
        let delta_secs = delta_time.as_secs_f32();
        
        self.x += self.velocity_x * delta_secs;
        self.y += self.velocity_y * delta_secs;
        self.velocity_y += 9.8 * delta_secs; // Gravity
        self.life -= delta_secs;
    }

    pub fn is_alive(&self) -> bool {
        self.life > 0.0
    }

    pub fn alpha(&self) -> f32 {
        (self.life / self.max_life).clamp(0.0, 1.0)
    }
}

/// Spinner animations for loading states
pub struct SpinnerAnimation {
    frames: Vec<&'static str>,
    current_frame: usize,
    frame_duration: Duration,
    elapsed: Duration,
}

impl SpinnerAnimation {
    pub fn dots() -> Self {
        Self {
            frames: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            current_frame: 0,
            frame_duration: Duration::from_millis(80),
            elapsed: Duration::ZERO,
        }
    }

    pub fn bouncing_ball() -> Self {
        Self {
            frames: vec!["⠁", "⠂", "⠄", "⡀", "⢀", "⠠", "⠐", "⠈"],
            current_frame: 0,
            frame_duration: Duration::from_millis(100),
            elapsed: Duration::ZERO,
        }
    }

    pub fn pulsing() -> Self {
        Self {
            frames: vec!["●", "◐", "◑", "◒", "◓", "◔", "◕", "◖", "◗"],
            current_frame: 0,
            frame_duration: Duration::from_millis(150),
            elapsed: Duration::ZERO,
        }
    }

    pub fn update(&mut self, delta_time: Duration) {
        self.elapsed += delta_time;
        
        if self.elapsed >= self.frame_duration {
            self.current_frame = (self.current_frame + 1) % self.frames.len();
            self.elapsed = Duration::ZERO;
        }
    }

    pub fn current_frame(&self) -> &str {
        self.frames[self.current_frame]
    }
}

/// Progress bar animation
pub struct ProgressAnimation {
    pub progress: AnimatedValue<f32>,
    pub pulse: AnimatedValue<f32>,
}

impl ProgressAnimation {
    pub fn new() -> Self {
        Self {
            progress: AnimatedValue::new(0.0),
            pulse: AnimatedValue::new(0.0),
        }
    }

    pub fn set_progress(&mut self, target: f32) {
        self.progress.animate_to(
            target.clamp(0.0, 1.0),
            Duration::from_millis(500),
            EasingFunction::EaseOut,
        );
    }

    pub fn start_pulse(&mut self) {
        self.pulse.animate_to(
            1.0,
            Duration::from_millis(800),
            EasingFunction::EaseInOut,
        );
    }

    pub fn update(&mut self, delta_time: Duration) {
        self.progress.update(delta_time);
        self.pulse.update(delta_time);
        
        // Auto-restart pulse animation
        if !self.pulse.is_animating() && *self.pulse.value() > 0.5 {
            self.pulse.animate_to(
                0.0,
                Duration::from_millis(800),
                EasingFunction::EaseInOut,
            );
        }
    }
}