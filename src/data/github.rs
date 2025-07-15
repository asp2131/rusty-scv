use anyhow::{Context, Result};
use chrono::{DateTime, Utc, Weekday, Duration, Datelike};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubCommit {
    pub sha: String,
    pub commit: CommitDetails,
    pub author: Option<GitHubUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitDetails {
    pub author: CommitAuthor,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitAuthor {
    pub name: String,
    pub email: String,
    pub date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub id: u64,
    pub avatar_url: String,
}

#[derive(Debug, Clone)]
pub struct WeekActivity {
    pub student_username: String,
    pub student_github_username: String,
    pub daily_commits: HashMap<Weekday, bool>, // true if committed on that day
    pub total_commits: usize,
    pub latest_commit: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

pub struct GitHubClient {
    client: reqwest::Client,
    token: Option<String>,
}

impl GitHubClient {
    pub fn new(token: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            token,
        }
    }

    pub async fn get_week_activity(&self, github_username: &str) -> Result<WeekActivity> {
        let weekdays = get_past_weekdays(5);
        let mut daily_commits = HashMap::new();
        let mut total_commits = 0;
        let mut latest_commit = None;

        // Initialize all weekdays to false
        for weekday in &weekdays {
            daily_commits.insert(*weekday, false);
        }

        match self.get_commits_for_user(github_username, &weekdays).await {
            Ok(commits) => {
                // Filter commits to only include those in the target weekdays
                let filtered_commits: Vec<_> = commits.into_iter()
                    .filter(|commit| {
                        let commit_weekday = commit.commit.author.date.weekday();
                        weekdays.contains(&commit_weekday)
                    })
                    .collect();
                
                total_commits = filtered_commits.len();
                
                // Process filtered commits to determine daily activity
                for commit in filtered_commits {
                    let commit_date = commit.commit.author.date;
                    let weekday = commit_date.weekday();
                    
                    // Mark this weekday as having commits
                    daily_commits.insert(weekday, true);
                    
                    // Update latest commit
                    if latest_commit.is_none() || commit_date > latest_commit.unwrap() {
                        latest_commit = Some(commit_date);
                    }
                }

                Ok(WeekActivity {
                    student_username: github_username.to_string(),
                    student_github_username: github_username.to_string(),
                    daily_commits,
                    total_commits,
                    latest_commit,
                    error: None,
                })
            }
            Err(e) => {
                // Return error activity with error message
                Ok(WeekActivity {
                    student_username: github_username.to_string(),
                    student_github_username: github_username.to_string(),
                    daily_commits,
                    total_commits: 0,
                    latest_commit: None,
                    error: Some(e.to_string()),
                })
            }
        }
    }

    async fn get_commits_for_user(&self, github_username: &str, weekdays: &[Weekday]) -> Result<Vec<GitHubCommit>> {
        let repo_name = format!("{}.github.io", github_username);
        let url = format!("https://api.github.com/repos/{}/{}/commits", github_username, repo_name);
        
        // Calculate the date range for the past 5 weekdays
        let start_date = get_earliest_weekday_date(weekdays);
        let end_date = Utc::now();
        
        let mut request = self.client.get(&url)
            .query(&[
                ("since", start_date.to_rfc3339()),
                ("until", end_date.to_rfc3339()),
                ("per_page", "100".to_string()),
            ]);

        // Add authorization header if token is available
        if let Some(token) = &self.token {
            request = request.header("Authorization", format!("token {}", token));
        }

        let response = request.send().await
            .with_context(|| format!("Failed to fetch commits for {}", github_username))?;

        if response.status().is_success() {
            let commits: Vec<GitHubCommit> = response.json().await
                .with_context(|| "Failed to parse GitHub API response")?;
            Ok(commits)
        } else if response.status() == 404 {
            // Repository not found - this is expected for some students
            Ok(Vec::new())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("GitHub API error {}: {}", status, error_text))
        }
    }

    pub async fn get_latest_activity(&self, github_username: &str) -> Result<Option<DateTime<Utc>>> {
        let repo_name = format!("{}.github.io", github_username);
        let url = format!("https://api.github.com/repos/{}/{}/commits", github_username, repo_name);
        
        let mut request = self.client.get(&url)
            .query(&[("per_page", "1")]);

        if let Some(token) = &self.token {
            request = request.header("Authorization", format!("token {}", token));
        }

        let response = request.send().await
            .with_context(|| format!("Failed to fetch latest commit for {}", github_username))?;

        if response.status().is_success() {
            let commits: Vec<GitHubCommit> = response.json().await
                .with_context(|| "Failed to parse GitHub API response")?;
            
            Ok(commits.first().map(|commit| commit.commit.author.date))
        } else if response.status() == 404 {
            // Repository not found
            Ok(None)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("GitHub API error {}: {}", status, error_text))
        }
    }
}

// Helper function to get the past N weekdays (Monday-Friday)
fn get_past_weekdays(count: usize) -> Vec<Weekday> {
    let mut weekdays = Vec::new();
    let mut current = Utc::now();
    
    while weekdays.len() < count {
        let weekday = current.weekday();
        if weekday != Weekday::Sat && weekday != Weekday::Sun {
            weekdays.push(weekday);
        }
        current = current - Duration::days(1);
    }
    
    weekdays.reverse(); // Return in chronological order
    weekdays
}

// Helper function to get the earliest date from weekdays
fn get_earliest_weekday_date(weekdays: &[Weekday]) -> DateTime<Utc> {
    let mut current = Utc::now();
    let mut days_back = 0;
    
    // Go back up to 7 days to find the earliest weekday
    while days_back < 7 {
        let weekday = current.weekday();
        if weekdays.contains(&weekday) {
            break;
        }
        current = current - Duration::days(1);
        days_back += 1;
    }
    
    // Go back additional days to cover all weekdays
    current - Duration::days(7)
}

// Helper function to format weekday for display
pub fn format_weekday(weekday: Weekday) -> &'static str {
    match weekday {
        Weekday::Mon => "Mon",
        Weekday::Tue => "Tue",
        Weekday::Wed => "Wed",
        Weekday::Thu => "Thu",
        Weekday::Fri => "Fri",
        Weekday::Sat => "Sat",
        Weekday::Sun => "Sun",
    }
}

// Helper function to get current weekdays for display
pub fn get_current_weekdays() -> Vec<Weekday> {
    vec![Weekday::Mon, Weekday::Tue, Weekday::Wed, Weekday::Thu, Weekday::Fri]
}