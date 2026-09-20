//! Buffer module - Rope-based text buffer for efficient editing

use ropey::Rope;
use std::path::PathBuf;

/// Represents a text buffer with efficient editing operations
#[derive(Debug, Clone)]
pub struct Buffer {
    /// The rope data structure for efficient text operations
    rope: Rope,
    /// File path if saved to disk
    path: Option<PathBuf>,
    /// Whether the buffer has unsaved changes
    is_dirty: bool,
    /// Cursor position (line, column)
    cursor: (usize, usize),
    /// Selection range (start, end)
    selection: Option<((usize, usize), (usize, usize))>,
}

impl Buffer {
    /// Create a new empty buffer
    pub fn new() -> Self {
        Self {
            rope: Rope::new(),
            path: None,
            is_dirty: false,
            cursor: (0, 0),
            selection: None,
        }
    }

    /// Create a buffer from existing text
    pub fn from_string(text: &str) -> Self {
        Self {
            rope: Rope::from_str(text),
            path: None,
            is_dirty: false,
            cursor: (0, 0),
            selection: None,
        }
    }

    /// Load a buffer from a file
    pub fn from_file(path: &PathBuf) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let mut buffer = Self::from_string(&content);
        buffer.path = Some(path.clone());
        Ok(buffer)
    }

    /// Get the text content as a String
    pub fn to_string(&self) -> String {
        self.rope.to_string()
    }

    /// Get a line by index
    pub fn line(&self, line_idx: usize) -> Option<String> {
        if line_idx < self.rope.len_lines() {
            Some(self.rope.line(line_idx).to_string())
        } else {
            None
        }
    }

    /// Get the number of lines
    pub fn len_lines(&self) -> usize {
        self.rope.len_lines()
    }

    /// Get the total number of characters
    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }

    /// Insert text at a position
    pub fn insert(&mut self, char_idx: usize, text: &str) {
        self.rope.insert(char_idx, text);
        self.is_dirty = true;
    }

    /// Delete a range of text
    pub fn delete(&mut self, start_char: usize, end_char: usize) {
        self.rope.remove(start_char..end_char);
        self.is_dirty = true;
    }

    /// Get character index from line/column
    pub fn line_col_to_char(&self, line: usize, col: usize) -> usize {
        let line_start = self.line_to_char_index(line);
        let line_end = if line + 1 < self.rope.len_lines() {
            self.line_to_char_index(line + 1)
        } else {
            self.rope.len_chars()
        };
        
        let max_col = line_end - line_start;
        line_start + col.min(max_col)
    }

    /// Get character index of the start of a line
    pub fn line_to_char_index(&self, line: usize) -> usize {
        self.rope.line_to_char(line.min(self.rope.len_lines().saturating_sub(1)))
    }

    /// Get line/column from character index
    pub fn char_to_line_col(&self, char_idx: usize) -> (usize, usize) {
        let line = self.rope.char_to_line(char_idx.min(self.rope.len_chars()));
        let line_start = self.rope.line_to_char(line);
        let col = char_idx - line_start;
        (line, col)
    }

    /// Set cursor position
    pub fn set_cursor(&mut self, line: usize, col: usize) {
        self.cursor = (line.min(self.rope.len_lines().saturating_sub(1)), col);
    }

    /// Get cursor position
    pub fn cursor(&self) -> (usize, usize) {
        self.cursor
    }

    /// Save the buffer to its file path
    pub fn save(&mut self) -> Result<(), std::io::Error> {
        if let Some(ref path) = self.path {
            std::fs::write(path, self.to_string())?;
            self.is_dirty = false;
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No file path set",
            ))
        }
    }

    /// Save the buffer to a specific path
    pub fn save_as(&mut self, path: PathBuf) -> Result<(), std::io::Error> {
        std::fs::write(&path, self.to_string())?;
        self.path = Some(path);
        self.is_dirty = false;
        Ok(())
    }

    /// Check if buffer has unsaved changes
    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    /// Get the file path
    pub fn path(&self) -> Option<&PathBuf> {
        self.path.as_ref()
    }

    /// Get the file name
    pub fn file_name(&self) -> Option<&str> {
        self.path.as_ref().and_then(|p| p.file_name().and_then(|n| n.to_str()))
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}
