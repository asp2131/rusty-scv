use anyhow::Result;
use rusqlite::{Connection, params};
use std::path::PathBuf;
use dirs::home_dir;
use chrono::{DateTime, Utc};

use super::models::{Class, Student};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub async fn init() -> Result<Self> {
        let db_path = get_database_path()?;
        
        let conn = Connection::open(&db_path)?;
        
        // Create tables if they don't exist
        Self::create_tables(&conn)?;
        
        Ok(Self { conn })
    }
    
    fn create_tables(conn: &Connection) -> Result<()> {
        // Create classes table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS classes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )
            "#,
            [],
        )?;
        
        // Create students table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS students (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                class_id INTEGER NOT NULL,
                username TEXT NOT NULL,
                github_username TEXT NOT NULL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (class_id) REFERENCES classes (id) ON DELETE CASCADE,
                UNIQUE(class_id, username)
            )
            "#,
            [],
        )?;
        
        // Create indexes
        conn.execute("CREATE INDEX IF NOT EXISTS idx_students_class_id ON students(class_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_students_username ON students(username)", [])?;
        
        Ok(())
    }
    
    // ===== CLASS OPERATIONS =====
    
    pub async fn create_class(&self, name: &str) -> Result<Class> {
        let mut stmt = self.conn.prepare(
            "INSERT INTO classes (name, created_at) VALUES (?, datetime('now')) RETURNING id, name, created_at"
        )?;
        
        let class = stmt.query_row(params![name], |row| {
            Ok(Class {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: Utc::now(), // For now, use current time
            })
        })?;
        
        Ok(class)
    }
    
    pub async fn get_classes(&self) -> Result<Vec<Class>> {
        let mut stmt = self.conn.prepare("SELECT id, name, created_at FROM classes ORDER BY name")?;
        let class_iter = stmt.query_map([], |row| {
            Ok(Class {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: Utc::now(), // For now, use current time
            })
        })?;
        
        let mut classes = Vec::new();
        for class in class_iter {
            classes.push(class?);
        }
        
        Ok(classes)
    }
    
    pub async fn get_class_by_id(&self, id: i64) -> Result<Option<Class>> {
        let mut stmt = self.conn.prepare("SELECT id, name, created_at FROM classes WHERE id = ?")?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(Class {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: Utc::now(), // For now, use current time
            })
        })?;
        
        match rows.next() {
            Some(class) => Ok(Some(class?)),
            None => Ok(None),
        }
    }
    
    pub async fn delete_class(&self, id: i64) -> Result<bool> {
        let affected = self.conn.execute("DELETE FROM classes WHERE id = ?", params![id])?;
        Ok(affected > 0)
    }
    
    // ===== STUDENT OPERATIONS =====
    
    pub async fn add_student(&self, class_id: i64, username: &str) -> Result<Student> {
        let mut stmt = self.conn.prepare(
            "INSERT INTO students (class_id, username, github_username, created_at) 
             VALUES (?, ?, ?, datetime('now')) 
             RETURNING id, class_id, username, github_username, created_at"
        )?;
        
        let student = stmt.query_row(params![class_id, username, username], |row| {
            Ok(Student {
                id: row.get(0)?,
                class_id: row.get(1)?,
                username: row.get(2)?,
                github_username: row.get(3)?,
                created_at: Utc::now(), // For now, use current time
            })
        })?;
        
        Ok(student)
    }
    
    pub async fn get_students_for_class(&self, class_id: i64) -> Result<Vec<Student>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, class_id, username, github_username, created_at 
             FROM students WHERE class_id = ? ORDER BY username"
        )?;
        let student_iter = stmt.query_map(params![class_id], |row| {
            Ok(Student {
                id: row.get(0)?,
                class_id: row.get(1)?,
                username: row.get(2)?,
                github_username: row.get(3)?,
                created_at: Utc::now(), // For now, use current time
            })
        })?;
        
        let mut students = Vec::new();
        for student in student_iter {
            students.push(student?);
        }
        
        Ok(students)
    }
    
    pub async fn delete_student(&self, id: i64) -> Result<bool> {
        let affected = self.conn.execute("DELETE FROM students WHERE id = ?", params![id])?;
        Ok(affected > 0)
    }
    
    pub async fn get_student_count_for_class(&self, class_id: i64) -> Result<i64> {
        let mut stmt = self.conn.prepare("SELECT COUNT(*) FROM students WHERE class_id = ?")?;
        let count: i64 = stmt.query_row(params![class_id], |row| row.get(0))?;
        Ok(count)
    }
}

fn get_database_path() -> Result<PathBuf> {
    let home = home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let scv_dir = home.join(".scv-rust"); // Different from Go version
    std::fs::create_dir_all(&scv_dir)?;
    
    let db_path = scv_dir.join("scv.db");
    Ok(db_path)
}

// Temporary init function for backwards compatibility
pub async fn init_db() -> Result<()> {
    let _db = Database::init().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_database_operations() -> Result<()> {
        let db = Database::init().await?;
        
        // Test class creation
        let class = db.create_class("Test Class").await?;
        assert_eq!(class.name, "Test Class");
        
        // Test student creation
        let student = db.add_student(class.id, "testuser").await?;
        assert_eq!(student.username, "testuser");
        assert_eq!(student.class_id, class.id);
        
        // Test getting students
        let students = db.get_students_for_class(class.id).await?;
        assert_eq!(students.len(), 1);
        
        // Test cleanup
        db.delete_student(student.id).await?;
        db.delete_class(class.id).await?;
        
        Ok(())
    }
}