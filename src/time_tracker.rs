//! Time tracking module - Tracks time spent on files and projects

use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Session data for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSession {
    /// Path to the file
    pub path: String,
    /// Total time spent in seconds
    pub total_seconds: u64,
    /// List of session start/end times
    pub sessions: Vec<(DateTime<Utc>, DateTime<Utc>)>,
    /// Last access time
    pub last_accessed: DateTime<Utc>,
}

/// Project-level time tracking data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectData {
    /// Project root path
    pub root_path: String,
    /// Files tracked in this project
    pub files: HashMap<String, FileSession>,
    /// Total project time in seconds
    pub total_seconds: u64,
    /// Project creation time
    pub created_at: DateTime<Utc>,
}

/// Main time tracker
pub struct TimeTracker {
    /// Currently active file
    current_file: Option<PathBuf>,
    /// Current session start time
    session_start: Option<DateTime<Utc>>,
    /// All tracked projects
    projects: HashMap<String, ProjectData>,
    /// Data file path for persistence
    data_path: PathBuf,
}

impl TimeTracker {
    /// Create a new time tracker
    pub fn new() -> Self {
        let data_path = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("aurora_ide")
            .join("time_tracking.json");
        
        Self {
            current_file: None,
            session_start: None,
            projects: HashMap::new(),
            data_path,
        }
    }

    /// Start tracking a file
    pub fn start_file(&mut self, path: PathBuf) {
        // End previous session if any
        self.end_current_session();
        
        self.current_file = Some(path.clone());
        self.session_start = Some(Utc::now());
        
        // Initialize project data if needed
        if let Some(project_root) = self.find_project_root(&path) {
            self.projects.entry(project_root.display().to_string())
                .or_insert_with(|| ProjectData {
                    root_path: project_root.display().to_string(),
                    files: HashMap::new(),
                    total_seconds: 0,
                    created_at: Utc::now(),
                });
        }
    }

    /// Stop tracking the current file
    pub fn end_current_session(&mut self) {
        if let (Some(path), Some(start)) = (self.current_file.take(), self.session_start.take()) {
            let end = Utc::now();
            let duration = end.signed_duration_since(start);
            
            if duration.num_seconds() > 0 {
                self.record_session(path, start, end, duration);
            }
        }
    }

    /// Record a session for a file
    fn record_session(&mut self, path: PathBuf, start: DateTime<Utc>, end: DateTime<Utc>, duration: Duration) {
        let path_str = path.to_string_lossy().to_string();
        
        if let Some(project_root) = self.find_project_root(&path) {
            if let Some(project) = self.projects.get_mut(&project_root.display().to_string()) {
                let seconds = duration.num_seconds() as u64;
                
                let file_session = project.files.entry(path_str.clone()).or_insert_with(|| FileSession {
                    path: path_str.clone(),
                    total_seconds: 0,
                    sessions: Vec::new(),
                    last_accessed: start,
                });
                
                file_session.total_seconds += seconds;
                file_session.sessions.push((start, end));
                file_session.last_accessed = end;
                
                project.total_seconds += seconds;
            }
        }
    }

    /// Find the project root (directory with .git or Cargo.toml)
    fn find_project_root(&self, path: &PathBuf) -> Option<PathBuf> {
        let mut current = path.parent()?.to_path_buf();
        
        loop {
            if current.join(".git").exists() || current.join("Cargo.toml").exists() {
                return Some(current);
            }
            
            if !current.pop() {
                break;
            }
        }
        
        // If no project root found, use the parent directory
        path.parent().map(|p| p.to_path_buf())
    }

    /// Get time spent on current file
    #[allow(dead_code)]
    pub fn get_current_file_time(&self) -> Option<u64> {
        self.current_file.as_ref().and_then(|path| {
            let path_str = path.to_string_lossy().to_string();
            
            if let Some(project_root) = self.find_project_root(path) {
                self.projects.get(&project_root.display().to_string())
                    .and_then(|p| p.files.get(&path_str))
                    .map(|f| f.total_seconds)
            } else {
                None
            }
        })
    }

    /// Get total project time
    #[allow(dead_code)]
    pub fn get_project_time(&self, path: &PathBuf) -> Option<u64> {
        self.find_project_root(path)
            .and_then(|root| self.projects.get(&root.display().to_string()))
            .map(|p| p.total_seconds)
    }

    /// Format seconds into human readable string
    #[allow(dead_code)]
    pub fn format_duration(seconds: u64) -> String {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;
        
        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, secs)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, secs)
        } else {
            format!("{}s", secs)
        }
    }

    /// Save tracking data to disk
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(self.data_path.parent().unwrap())?;
        let json = serde_json::to_string_pretty(&self.projects)?;
        std::fs::write(&self.data_path, json)?;
        Ok(())
    }

    /// Load tracking data from disk
    pub fn load(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.data_path.exists() {
            let json = std::fs::read_to_string(&self.data_path)?;
            self.projects = serde_json::from_str(&json)?;
        }
        Ok(())
    }

    /// Get statistics for all files in a project
    #[allow(dead_code)]
    pub fn get_project_stats(&self, project_root: &str) -> Option<&ProjectData> {
        self.projects.get(project_root)
    }
}

impl Default for TimeTracker {
    fn default() -> Self {
        Self::new()
    }
}
