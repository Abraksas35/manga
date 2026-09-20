//! Checkpoint module - Manages checkpoint saves (last 10 checkpoints)

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::collections::VecDeque;

/// A checkpoint save
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Unique identifier
    pub id: u64,
    /// Timestamp of the checkpoint
    pub timestamp: DateTime<Utc>,
    /// File path
    pub file_path: String,
    /// Code content at checkpoint
    pub content: String,
    /// Optional description/note
    pub description: Option<String>,
    /// Cursor position at time of checkpoint
    pub cursor_line: usize,
    pub cursor_col: usize,
}

/// Checkpoint manager for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCheckpoints {
    /// Path to the file
    pub file_path: String,
    /// Queue of checkpoints (max 10)
    pub checkpoints: VecDeque<Checkpoint>,
    /// Maximum number of checkpoints to keep
    pub max_checkpoints: usize,
    /// Counter for generating unique IDs
    pub next_id: u64,
}

impl FileCheckpoints {
    /// Create a new checkpoint manager for a file
    pub fn new(file_path: String) -> Self {
        Self {
            file_path,
            checkpoints: VecDeque::new(),
            max_checkpoints: 10, // As requested: 10 last checkpoints
            next_id: 0,
        }
    }

    /// Add a new checkpoint
    pub fn add_checkpoint(&mut self, content: String, description: Option<String>, cursor: (usize, usize)) {
        let checkpoint = Checkpoint {
            id: self.next_id,
            timestamp: Utc::now(),
            file_path: self.file_path.clone(),
            content,
            description,
            cursor_line: cursor.0,
            cursor_col: cursor.1,
        };

        self.next_id += 1;
        self.checkpoints.push_back(checkpoint);

        // Remove oldest if we exceed the limit
        while self.checkpoints.len() > self.max_checkpoints {
            self.checkpoints.pop_front();
        }
    }

    /// Get the latest checkpoint
    pub fn get_latest(&self) -> Option<&Checkpoint> {
        self.checkpoints.back()
    }

    /// Get checkpoint by index (0 = oldest, len-1 = newest)
    pub fn get_by_index(&self, index: usize) -> Option<&Checkpoint> {
        self.checkpoints.get(index)
    }

    /// Get all checkpoints
    pub fn get_all(&self) -> &VecDeque<Checkpoint> {
        &self.checkpoints
    }

    /// Get checkpoint by ID
    pub fn get_by_id(&self, id: u64) -> Option<&Checkpoint> {
        self.checkpoints.iter().find(|c| c.id == id)
    }

    /// Restore to a specific checkpoint (returns the content)
    pub fn restore_to_checkpoint(&self, id: u64) -> Option<(String, (usize, usize))> {
        self.get_by_id(id).map(|c| (c.content.clone(), (c.cursor_line, c.cursor_col)))
    }

    /// Delete a checkpoint by ID
    pub fn delete_checkpoint(&mut self, id: u64) -> bool {
        if let Some(pos) = self.checkpoints.iter().position(|c| c.id == id) {
            self.checkpoints.remove(pos);
            true
        } else {
            false
        }
    }

    /// Clear all checkpoints
    pub fn clear(&mut self) {
        self.checkpoints.clear();
    }
}

/// Global checkpoint manager for all files
pub struct CheckpointManager {
    /// Checkpoint managers for individual files
    file_checkpoints: std::collections::HashMap<String, FileCheckpoints>,
    /// Directory for storing checkpoint data
    data_dir: PathBuf,
}

impl CheckpointManager {
    /// Create a new checkpoint manager
    pub fn new() -> Self {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("aurora_ide")
            .join("checkpoints");

        Self {
            file_checkpoints: std::collections::HashMap::new(),
            data_dir,
        }
    }

    /// Get or create checkpoint manager for a file
    pub fn get_or_create(&mut self, file_path: &PathBuf) -> &mut FileCheckpoints {
        let path_str = file_path.to_string_lossy().to_string();
        
        self.file_checkpoints.entry(path_str.clone()).or_insert_with(|| {
            FileCheckpoints::new(path_str)
        })
    }

    /// Create a checkpoint for a file
    pub fn create_checkpoint(&mut self, file_path: &PathBuf, content: String, description: Option<String>, cursor: (usize, usize)) {
        let fc = self.get_or_create(file_path);
        fc.add_checkpoint(content, description, cursor);
    }

    /// Get all checkpoints for a file
    pub fn get_checkpoints(&self, file_path: &PathBuf) -> Option<&FileCheckpoints> {
        let path_str = file_path.to_string_lossy().to_string();
        self.file_checkpoints.get(&path_str)
    }

    /// Restore a file to a specific checkpoint
    pub fn restore_checkpoint(&self, file_path: &PathBuf, checkpoint_id: u64) -> Option<(String, (usize, usize))> {
        let path_str = file_path.to_string_lossy().to_string();
        self.file_checkpoints.get(&path_str)
            .and_then(|fc| fc.restore_to_checkpoint(checkpoint_id))
    }

    /// List all files with checkpoints
    pub fn list_files_with_checkpoints(&self) -> Vec<&String> {
        self.file_checkpoints.keys().collect()
    }

    /// Save all checkpoints to disk
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&self.data_dir)?;
        
        for (path, fc) in &self.file_checkpoints {
            let safe_path = path.replace('/', "_").replace('\\', "_").replace(':', "_");
            let file_path = self.data_dir.join(format!("{}.json", safe_path));
            
            let json = serde_json::to_string_pretty(fc)?;
            std::fs::write(file_path, json)?;
        }
        
        Ok(())
    }

    /// Load all checkpoints from disk
    pub fn load(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.data_dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(&self.data_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let json = std::fs::read_to_string(&path)?;
                let fc: FileCheckpoints = serde_json::from_str(&json)?;
                
                self.file_checkpoints.insert(fc.file_path.clone(), fc);
            }
        }

        Ok(())
    }

    /// Get total number of checkpoints across all files
    pub fn total_checkpoints(&self) -> usize {
        self.file_checkpoints.values().map(|fc| fc.checkpoints.len()).sum()
    }

    /// Get checkpoint count for a specific file
    pub fn checkpoint_count(&self, file_path: &PathBuf) -> usize {
        let path_str = file_path.to_string_lossy().to_string();
        self.file_checkpoints.get(&path_str)
            .map(|fc| fc.checkpoints.len())
            .unwrap_or(0)
    }
}

impl Default for CheckpointManager {
    fn default() -> Self {
        Self::new()
    }
}
