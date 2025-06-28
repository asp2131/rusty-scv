use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Class {
    pub id: i64,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Student {
    pub id: i64,
    pub class_id: i64,
    pub username: String,
    pub github_username: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentWithClass {
    pub student: Student,
    pub class: Class,
}

impl Class {
    pub fn new(name: String) -> Self {
        Self {
            id: 0, // Will be set by database
            name,
            created_at: Utc::now(),
        }
    }
}

impl Student {
    pub fn new(class_id: i64, username: String) -> Self {
        Self {
            id: 0, // Will be set by database
            class_id,
            github_username: username.clone(),
            username,
            created_at: Utc::now(),
        }
    }
}