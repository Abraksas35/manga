//! History module - Code timeline with slider to view changes over time

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::path::PathBuf;

/// A snapshot of code at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSnapshot {
    /// Timestamp of the snapshot
    pub timestamp: DateTime<Utc>,
    /// The code content
    pub content: String,
    /// Type of change
    pub change_type: ChangeType,
    /// Optional description
    pub description: Option<String>,
}

/// Type of change that triggered a snapshot
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeType {
    /// Manual save (Ctrl+S)
    ManualSave,
    /// Checkpoint save (Alt+Shift+S)
    Checkpoint,
    /// Auto-save
    AutoSave,
    /// Significant edit (many lines changed)
    SignificantEdit,
    /// Project growth milestone
    ProjectMilestone,
}

/// History data for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHistory {
    /// Path to the file
    pub path: String,
    /// List of snapshots in chronological order
    pub snapshots: Vec<CodeSnapshot>,
    /// Maximum number of snapshots to keep
    pub max_snapshots: usize,
}

impl FileHistory {
    /// Create a new file history
    pub fn new(path: String, max_snapshots: usize) -> Self {
        Self {
            path,
            snapshots: Vec::new(),
            max_snapshots,
        }
    }

    /// Add a new snapshot
    pub fn add_snapshot(&mut self, content: String, change_type: ChangeType, description: Option<String>) {
        let snapshot = CodeSnapshot {
            timestamp: Utc::now(),
            content,
            change_type,
            description,
        };

        self.snapshots.push(snapshot);

        // Trim old snapshots if we exceed the limit
        // But always keep at least one checkpoint if available
        if self.snapshots.len() > self.max_snapshots {
            // Find the first checkpoint index
            let first_checkpoint = self.snapshots.iter()
                .position(|s| s.change_type == ChangeType::Checkpoint);
            
            // If we have checkpoints and the first one is at index 0, keep it
            if let Some(idx) = first_checkpoint {
                if idx == 0 && self.snapshots.len() > 1 {
                    // Remove the second oldest instead of the oldest
                    self.snapshots.remove(1);
                } else {
                    self.snapshots.remove(0);
                }
            } else {
                self.snapshots.remove(0);
            }
        }
    }

    /// Get snapshot at a specific time (closest before or at the time)
    pub fn get_snapshot_at(&self, time: DateTime<Utc>) -> Option<&CodeSnapshot> {
        self.snapshots
            .iter()
            .rev()
            .find(|s| s.timestamp <= time)
    }

    /// Get snapshot by index (0 = oldest, len-1 = newest)
    pub fn get_snapshot_by_index(&self, index: usize) -> Option<&CodeSnapshot> {
        self.snapshots.get(index)
    }

    /// Get all snapshots between two times
    pub fn get_snapshots_between(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<&CodeSnapshot> {
        self.snapshots
            .iter()
            .filter(|s| s.timestamp >= start && s.timestamp <= end)
            .collect()
    }

    /// Get the latest snapshot
    pub fn get_latest(&self) -> Option<&CodeSnapshot> {
        self.snapshots.last()
    }

    /// Get the first snapshot
    pub fn get_first(&self) -> Option<&CodeSnapshot> {
        self.snapshots.first()
    }

    /// Get total code growth (lines added over time)
    pub fn get_growth_timeline(&self) -> Vec<(DateTime<Utc>, usize)> {
        self.snapshots
            .iter()
            .map(|s| (s.timestamp, s.content.lines().count()))
            .collect()
    }

    /// Get only checkpoint snapshots
    pub fn get_checkpoints(&self) -> Vec<&CodeSnapshot> {
        self.snapshots
            .iter()
            .filter(|s| s.change_type == ChangeType::Checkpoint)
            .collect()
    }
}

/// Main history manager for the IDE
pub struct HistoryManager {
    /// Histories for all open files
    file_histories: std::collections::HashMap<String, FileHistory>,
    /// Maximum snapshots per file
    max_snapshots_per_file: usize,
    /// Data directory for persistence
    data_dir: PathBuf,
}

impl HistoryManager {
    /// Create a new history manager
    pub fn new() -> Self {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("aurora_ide")
            .join("history");

        Self {
            file_histories: std::collections::HashMap::new(),
            max_snapshots_per_file: 100, // Default: 100 snapshots per file
            data_dir,
        }
    }

    /// Get or create history for a file
    pub fn get_or_create_history(&mut self, path: &PathBuf) -> &mut FileHistory {
        let path_str = path.to_string_lossy().to_string();
        
        self.file_histories.entry(path_str.clone()).or_insert_with(|| {
            FileHistory::new(path_str, self.max_snapshots_per_file)
        })
    }

    /// Record a code change
    pub fn record_change(&mut self, path: &PathBuf, content: String, change_type: ChangeType, description: Option<String>) {
        let history = self.get_or_create_history(path);
        history.add_snapshot(content, change_type, description);
    }

    /// Get code at a specific time for a file
    pub fn get_code_at_time(&self, path: &PathBuf, time: DateTime<Utc>) -> Option<String> {
        let path_str = path.to_string_lossy().to_string();
        
        self.file_histories.get(&path_str)
            .and_then(|h| h.get_snapshot_at(time))
            .map(|s| s.content.clone())
    }

    /// Get growth timeline for a file
    pub fn get_file_growth(&self, path: &PathBuf) -> Option<Vec<(DateTime<Utc>, usize)>> {
        let path_str = path.to_string_lossy().to_string();
        
        self.file_histories.get(&path_str)
            .map(|h| h.get_growth_timeline())
    }

    /// Get project-wide growth (sum of all files)
    pub fn get_project_growth(&self, project_root: &PathBuf) -> Vec<(DateTime<Utc>, usize)> {
        let mut timeline: Vec<(DateTime<Utc>, usize)> = Vec::new();
        let project_str = project_root.to_string_lossy().to_string();

        for (path, history) in &self.file_histories {
            if path.starts_with(&project_str) {
                for (time, lines) in history.get_growth_timeline() {
                    // Find or create entry for this timestamp
                    if let Some(entry) = timeline.iter_mut().find(|(t, _)| *t == time) {
                        entry.1 += lines;
                    } else {
                        timeline.push((time, lines));
                    }
                }
            }
        }

        timeline.sort_by_key(|(time, _)| *time);
        timeline
    }

    /// Save all histories to disk
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&self.data_dir)?;
        
        for (path, history) in &self.file_histories {
            let safe_path = path.replace('/', "_").replace('\\', "_").replace(':', "_");
            let file_path = self.data_dir.join(format!("{}.json", safe_path));
            
            let json = serde_json::to_string_pretty(history)?;
            std::fs::write(file_path, json)?;
        }
        
        Ok(())
    }

    /// Load all histories from disk
    pub fn load(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.data_dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(&self.data_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let json = std::fs::read_to_string(&path)?;
                let history: FileHistory = serde_json::from_str(&json)?;
                
                // Convert safe path back to original (approximate)
                self.file_histories.insert(history.path.clone(), history);
            }
        }

        Ok(())
    }

    /// Set maximum snapshots per file
    pub fn set_max_snapshots(&mut self, max: usize) {
        self.max_snapshots_per_file = max;
        
        // Update existing histories
        for history in self.file_histories.values_mut() {
            history.max_snapshots = max;
        }
    }
}

impl Default for HistoryManager {
    fn default() -> Self {
        Self::new()
    }
}
