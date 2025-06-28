use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use dirs::home_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub theme: String,
    pub github_token: Option<String>,
    pub animation_speed: f32,
    pub enable_particle_effects: bool,
    pub frame_rate: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "neon_night".to_string(),
            github_token: None,
            animation_speed: 1.0,
            enable_particle_effects: true,
            frame_rate: 60,
        }
    }
}

impl Config {
    pub async fn load() -> Result<Self> {
        let config_path = get_config_path()?;
        
        if config_path.exists() {
            let contents = tokio::fs::read_to_string(config_path).await?;
            let config: Config = serde_json::from_str(&contents)?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save().await?;
            Ok(config)
        }
    }
    
    pub async fn save(&self) -> Result<()> {
        let config_path = get_config_path()?;
        let contents = serde_json::to_string_pretty(self)?;
        tokio::fs::write(config_path, contents).await?;
        Ok(())
    }
}

fn get_config_path() -> Result<PathBuf> {
    let home = home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let scv_dir = home.join(".scv-rust");
    std::fs::create_dir_all(&scv_dir)?;
    Ok(scv_dir.join("config.json"))
}