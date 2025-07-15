use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;
use tokio::process::Command as TokioCommand;

pub struct GitManager {
    pub repos_dir: PathBuf,
}

impl GitManager {
    pub fn new(repos_dir: PathBuf) -> Self {
        Self { repos_dir }
    }

    pub async fn clone_repo(&self, github_username: &str, class_name: &str) -> Result<()> {
        let repo_url = format!("https://github.com/{}/{}.github.io.git", github_username, github_username);
        let repo_path = self.repos_dir.join(class_name).join(github_username);
        
        if repo_path.exists() {
            return Err(anyhow::anyhow!("Repository already exists at {}", repo_path.display()));
        }

        std::fs::create_dir_all(&repo_path.parent().unwrap())?;

        let output = TokioCommand::new("git")
            .arg("clone")
            .arg(&repo_url)
            .arg(&repo_path)
            .output()
            .await
            .context("Failed to execute git clone command")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Git clone failed: {}", error));
        }

        Ok(())
    }

    pub async fn pull_repo(&self, github_username: &str, class_name: &str) -> Result<()> {
        let repo_path = self.repos_dir.join(class_name).join(github_username);
        
        if !repo_path.exists() {
            return Err(anyhow::anyhow!("Repository not found at {}", repo_path.display()));
        }

        let output = TokioCommand::new("git")
            .arg("pull")
            .arg("origin")
            .arg("main")
            .current_dir(&repo_path)
            .output()
            .await
            .context("Failed to execute git pull command")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Git pull failed: {}", error));
        }

        Ok(())
    }

    pub async fn clean_repo(&self, github_username: &str, class_name: &str) -> Result<()> {
        let repo_path = self.repos_dir.join(class_name).join(github_username);
        
        if !repo_path.exists() {
            return Err(anyhow::anyhow!("Repository not found at {}", repo_path.display()));
        }

        let output = TokioCommand::new("git")
            .arg("reset")
            .arg("--hard")
            .arg("HEAD")
            .current_dir(&repo_path)
            .output()
            .await
            .context("Failed to execute git reset command")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Git reset failed: {}", error));
        }

        let output = TokioCommand::new("git")
            .arg("clean")
            .arg("-fd")
            .current_dir(&repo_path)
            .output()
            .await
            .context("Failed to execute git clean command")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Git clean failed: {}", error));
        }

        Ok(())
    }

    pub fn open_terminal(&self, github_username: &str, class_name: &str) -> Result<()> {
        let repo_path = self.repos_dir.join(class_name).join(github_username);
        
        if !repo_path.exists() {
            return Err(anyhow::anyhow!("Repository not found at {}", repo_path.display()));
        }

        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .arg("-a")
                .arg("Terminal")
                .arg(&repo_path)
                .spawn()
                .context("Failed to open terminal")?;
        }

        #[cfg(target_os = "linux")]
        {
            Command::new("gnome-terminal")
                .arg("--working-directory")
                .arg(&repo_path)
                .spawn()
                .context("Failed to open terminal")?;
        }

        #[cfg(target_os = "windows")]
        {
            Command::new("cmd")
                .arg("/C")
                .arg("start")
                .arg("cmd")
                .arg("/K")
                .arg(format!("cd /d {}", repo_path.display()))
                .spawn()
                .context("Failed to open terminal")?;
        }

        Ok(())
    }

    pub fn get_repo_path(&self, github_username: &str, class_name: &str) -> PathBuf {
        self.repos_dir.join(class_name).join(github_username)
    }

    pub fn repo_exists(&self, github_username: &str, class_name: &str) -> bool {
        self.get_repo_path(github_username, class_name).exists()
    }

    pub async fn clone_all_repos(&self, students: &[crate::data::Student], class_name: &str) -> Result<Vec<(String, Result<()>)>> {
        let mut results = Vec::new();
        
        for student in students {
            let result = self.clone_repo(&student.github_username, class_name).await;
            results.push((student.github_username.clone(), result));
        }
        
        Ok(results)
    }
}