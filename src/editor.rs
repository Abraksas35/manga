//! Editor module - Main text editor component

use crate::buffer::Buffer;
use crate::syntax_highlighter::{SyntaxHighlighter, HighlightedSegment};
use crate::history::{HistoryManager, ChangeType};
use crate::checkpoint::CheckpointManager;
use winit::keyboard::{KeyCode, ModifiersState};
use std::path::PathBuf;

/// Viewport for visible portion of the editor
#[derive(Debug, Clone)]
pub struct Viewport {
    /// First visible line
    pub scroll_y: usize,
    /// First visible column (for horizontal scroll)
    pub scroll_x: usize,
    /// Number of visible lines
    pub visible_lines: usize,
    /// Number of visible columns
    pub visible_columns: usize,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            scroll_y: 0,
            scroll_x: 0,
            visible_lines: 50,
            visible_columns: 120,
        }
    }
}

/// Editor state and operations
pub struct Editor {
    /// Current buffer
    buffer: Buffer,
    /// Syntax highlighter
    highlighter: SyntaxHighlighter,
    /// Viewport settings
    viewport: Viewport,
    /// History manager
    history: HistoryManager,
    /// Checkpoint manager
    checkpoints: CheckpointManager,
    /// Clipboard content
    clipboard: String,
    /// Whether to show line numbers
    show_line_numbers: bool,
    /// Tab width in spaces
    tab_width: usize,
    /// Current IME composition
    ime_composition: Option<String>,
}

impl Editor {
    /// Create a new editor
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(),
            highlighter: SyntaxHighlighter::new(),
            viewport: Viewport::default(),
            history: HistoryManager::new(),
            checkpoints: CheckpointManager::new(),
            clipboard: String::new(),
            show_line_numbers: true,
            tab_width: 4,
            ime_composition: None,
        }
    }

    /// Open a file
    pub fn open_file(&mut self, path: &PathBuf) -> Result<(), std::io::Error> {
        self.buffer = Buffer::from_file(path)?;
        
        // Set up syntax highlighting based on extension
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let _ = self.highlighter.set_language_from_extension(ext);
        }
        
        // Record in history
        self.history.record_change(
            path,
            self.buffer.to_string(),
            ChangeType::ManualSave,
            Some("File opened".to_string()),
        );
        
        Ok(())
    }

    /// Create a new empty file
    pub fn new_file(&mut self) {
        self.buffer = Buffer::new();
    }

    /// Save the current file
    pub fn save(&mut self) -> Result<(), std::io::Error> {
        self.buffer.save()?;
        
        // Record in history
        if let Some(path) = self.buffer.path() {
            self.history.record_change(
                path,
                self.buffer.to_string(),
                ChangeType::ManualSave,
                None,
            );
        }
        
        Ok(())
    }

    /// Save as checkpoint (Alt+Shift+S)
    pub fn save_checkpoint(&mut self, description: Option<String>) -> Result<(), std::io::Error> {
        // First save the file
        self.save()?;
        
        // Then create a checkpoint
        if let Some(path) = self.buffer.path() {
            let cursor = self.buffer.cursor();
            self.checkpoints.create_checkpoint(
                path,
                self.buffer.to_string(),
                description.clone(),
                cursor,
            );
            
            // Also record in history
            self.history.record_change(
                path,
                self.buffer.to_string(),
                ChangeType::Checkpoint,
                description,
            );
        }
        
        Ok(())
    }

    /// Handle keyboard input
    pub fn handle_key(&mut self, key: KeyCode, modifiers: ModifiersState) -> bool {
        // Return true if the key was handled
        
        // Ctrl+S - Save
        if modifiers.contains(ModifiersState::CONTROL) && !modifiers.contains(ModifiersState::SHIFT) && !modifiers.contains(ModifiersState::ALT) && key == KeyCode::KeyS {
            let _ = self.save();
            return true;
        }
        
        // Alt+Shift+S - Checkpoint save
        if modifiers.contains(ModifiersState::ALT) && modifiers.contains(ModifiersState::SHIFT) && !modifiers.contains(ModifiersState::CONTROL) && key == KeyCode::KeyS {
            let _ = self.save_checkpoint(None);
            return true;
        }
        
        // Ctrl+Z - Undo (basic implementation)
        if modifiers.contains(ModifiersState::CONTROL) && !modifiers.contains(ModifiersState::SHIFT) && !modifiers.contains(ModifiersState::ALT) && key == KeyCode::KeyZ {
            // TODO: Implement proper undo
            return true;
        }
        
        // Ctrl+Y - Redo
        if modifiers.contains(ModifiersState::CONTROL) && !modifiers.contains(ModifiersState::SHIFT) && !modifiers.contains(ModifiersState::ALT) && key == KeyCode::KeyY {
            // TODO: Implement proper redo
            return true;
        }
        
        // Ctrl+A - Select all
        if modifiers.contains(ModifiersState::CONTROL) && !modifiers.contains(ModifiersState::SHIFT) && !modifiers.contains(ModifiersState::ALT) && key == KeyCode::KeyA {
            // Select all logic would go here
            return true;
        }
        
        // Ctrl+F - Find
        if modifiers.contains(ModifiersState::CONTROL) && !modifiers.contains(ModifiersState::SHIFT) && !modifiers.contains(ModifiersState::ALT) && key == KeyCode::KeyF {
            // TODO: Open find dialog
            return true;
        }
        
        // Ctrl+H - Replace
        if modifiers.contains(ModifiersState::CONTROL) && !modifiers.contains(ModifiersState::SHIFT) && !modifiers.contains(ModifiersState::ALT) && key == KeyCode::KeyH {
            // TODO: Open replace dialog
            return true;
        }
        
        // Ctrl+G - Go to line
        if modifiers.contains(ModifiersState::CONTROL) && !modifiers.contains(ModifiersState::SHIFT) && !modifiers.contains(ModifiersState::ALT) && key == KeyCode::KeyG {
            // TODO: Open go to line dialog
            return true;
        }
        
        // Ctrl+/ - Toggle comment
        if modifiers.contains(ModifiersState::CONTROL) && !modifiers.contains(ModifiersState::SHIFT) && !modifiers.contains(ModifiersState::ALT) && key == KeyCode::Slash {
            self.toggle_comment();
            return true;
        }
        
        // Tab handling
        if key == KeyCode::Tab && !modifiers.contains(ModifiersState::CONTROL) {
            if modifiers.contains(ModifiersState::SHIFT) {
                self.outdent_selection();
            } else {
                self.indent_selection();
            }
            return true;
        }
        
        // Navigation keys
        match key {
            KeyCode::ArrowLeft => {
                self.move_cursor(-1, 0, modifiers);
                return true;
            }
            KeyCode::ArrowRight => {
                self.move_cursor(1, 0, modifiers);
                return true;
            }
            KeyCode::ArrowUp => {
                self.move_cursor(0, -1, modifiers);
                return true;
            }
            KeyCode::ArrowDown => {
                self.move_cursor(0, 1, modifiers);
                return true;
            }
            KeyCode::Home => {
                self.go_to_line_start(modifiers);
                return true;
            }
            KeyCode::End => {
                self.go_to_line_end(modifiers);
                return true;
            }
            KeyCode::PageUp => {
                self.page_up();
                return true;
            }
            KeyCode::PageDown => {
                self.page_down();
                return true;
            }
            _ => {}
        }
        
        false
    }

    /// Handle text input
    pub fn handle_text(&mut self, text: &str) {
        let (line, col) = self.buffer.cursor();
        let char_idx = self.buffer.line_col_to_char(line, col);
        
        self.buffer.insert(char_idx, text);
        
        // Update cursor position
        let new_col = col + text.chars().count();
        self.buffer.set_cursor(line, new_col);
        
        // Record in history periodically
        if let Some(path) = self.buffer.path() {
            // Only record significant changes to avoid too many snapshots
            static CHANGE_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            let count = CHANGE_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            
            if count % 100 == 0 {
                self.history.record_change(
                    path,
                    self.buffer.to_string(),
                    ChangeType::SignificantEdit,
                    None,
                );
            }
        }
    }

    /// Move cursor
    fn move_cursor(&mut self, dx: i32, dy: i32, _modifiers: ModifiersState) {
        let (mut line, mut col) = self.buffer.cursor();
        
        if dx < 0 && col > 0 {
            col -= 1;
        } else if dx > 0 {
            if let Some(line_content) = self.buffer.line(line) {
                if col < line_content.chars().count() {
                    col += 1;
                }
            }
        }
        
        if dy < 0 && line > 0 {
            line -= 1;
        } else if dy > 0 {
            if line + 1 < self.buffer.len_lines() {
                line += 1;
            }
        }
        
        self.buffer.set_cursor(line, col);
        
        // Update viewport if needed
        self.update_viewport();
    }

    /// Go to line start
    fn go_to_line_start(&mut self, _modifiers: ModifiersState) {
        let (line, _) = self.buffer.cursor();
        self.buffer.set_cursor(line, 0);
    }

    /// Go to line end
    fn go_to_line_end(&mut self, _modifiers: ModifiersState) {
        let (line, _) = self.buffer.cursor();
        if let Some(line_content) = self.buffer.line(line) {
            let col = line_content.chars().count();
            self.buffer.set_cursor(line, col);
        }
    }

    /// Page up
    fn page_up(&mut self) {
        let (_, col) = self.buffer.cursor();
        let new_line = self.viewport.scroll_y.saturating_sub(self.viewport.visible_lines);
        self.buffer.set_cursor(new_line, col);
        self.viewport.scroll_y = new_line;
    }

    /// Page down
    fn page_down(&mut self) {
        let (_, col) = self.buffer.cursor();
        let new_line = (self.viewport.scroll_y + self.viewport.visible_lines * 2)
            .min(self.buffer.len_lines().saturating_sub(1));
        self.buffer.set_cursor(new_line, col);
        self.viewport.scroll_y = self.viewport.scroll_y + self.viewport.visible_lines;
    }

    /// Indent selection
    fn indent_selection(&mut self) {
        let spaces = " ".repeat(self.tab_width);
        let (line, col) = self.buffer.cursor();
        let char_idx = self.buffer.line_col_to_char(line, col);
        
        self.buffer.insert(char_idx, &spaces);
        self.buffer.set_cursor(line, col + self.tab_width);
    }

    /// Outdent selection
    fn outdent_selection(&mut self) {
        let (line, col) = self.buffer.cursor();
        
        if col >= self.tab_width {
            let start_idx = self.buffer.line_col_to_char(line, col - self.tab_width);
            let end_idx = self.buffer.line_col_to_char(line, col);
            
            // Check if the characters before cursor are spaces
            let check_str = self.buffer.to_string();
            let chars: Vec<char> = check_str.chars().collect();
            
            let is_spaces = (start_idx..end_idx).all(|i| {
                i < chars.len() && (chars[i] == ' ' || chars[i] == '\t')
            });
            
            if is_spaces {
                self.buffer.delete(start_idx, end_idx);
                self.buffer.set_cursor(line, col - self.tab_width);
            }
        }
    }

    /// Toggle comment on current line(s)
    fn toggle_comment(&mut self) {
        // Basic implementation - would need language-specific comment prefixes
        let (line, col) = self.buffer.cursor();
        
        if let Some(mut line_content) = self.buffer.line(line) {
            let trimmed = line_content.trim_start();
            
            if trimmed.starts_with("//") {
                // Remove comment
                let prefix_len = line_content.len() - trimmed.len();
                line_content.replace_range(prefix_len..prefix_len + 2, "");
            } else {
                // Add comment
                line_content.insert_str(0, "// ");
            }
            
            // Replace the line
            let line_start = self.buffer.line_to_char_index(line);
            let line_end = if line + 1 < self.buffer.len_lines() {
                self.buffer.line_to_char_index(line + 1)
            } else {
                self.buffer.len_chars()
            };
            
            self.buffer.delete(line_start, line_end);
            self.buffer.insert(line_start, &line_content);
            if !line_content.ends_with('\n') {
                self.buffer.insert(line_start + line_content.len(), "\n");
            }
            
            self.buffer.set_cursor(line, col);
        }
    }

    /// Set viewport size
    pub fn set_viewport_size(&mut self, visible_lines: usize, visible_columns: usize) {
        self.viewport.visible_lines = visible_lines;
        self.viewport.visible_columns = visible_columns;
    }

    /// Update viewport based on cursor position
    fn update_viewport(&mut self) {
        let (line, _) = self.buffer.cursor();
        
        // Ensure cursor is visible
        if line < self.viewport.scroll_y {
            self.viewport.scroll_y = line;
        } else if line >= self.viewport.scroll_y + self.viewport.visible_lines {
            self.viewport.scroll_y = line - self.viewport.visible_lines + 1;
        }
    }

    /// Get highlighted segments for rendering
    pub fn get_highlights(&self) -> Vec<HighlightedSegment> {
        self.highlighter.highlight(&self.buffer.to_string())
    }

    /// Get the current buffer
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    /// Get mutable reference to buffer
    pub fn buffer_mut(&mut self) -> &mut Buffer {
        &mut self.buffer
    }

    /// Get viewport
    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }

    /// Get mutable reference to viewport
    pub fn viewport_mut(&mut self) -> &mut Viewport {
        &mut self.viewport
    }

    /// Scroll horizontally (for Ctrl+Scroll wheel)
    pub fn scroll_horizontal(&mut self, delta: i32) {
        let max_scroll = self.viewport.visible_columns;
        
        if delta > 0 {
            self.viewport.scroll_x = (self.viewport.scroll_x + delta as usize).min(max_scroll);
        } else {
            self.viewport.scroll_x = self.viewport.scroll_x.saturating_sub((-delta) as usize);
        }
    }

    /// Get syntax highlighter
    pub fn highlighter(&self) -> &SyntaxHighlighter {
        &self.highlighter
    }

    /// Get mutable reference to highlighter
    pub fn highlighter_mut(&mut self) -> &mut SyntaxHighlighter {
        &mut self.highlighter
    }

    /// Get history manager
    pub fn history(&self) -> &HistoryManager {
        &self.history
    }

    /// Get mutable reference to history manager
    pub fn history_mut(&mut self) -> &mut HistoryManager {
        &mut self.history
    }

    /// Get checkpoint manager
    pub fn checkpoints(&self) -> &CheckpointManager {
        &self.checkpoints
    }

    /// Get mutable reference to checkpoint manager
    pub fn checkpoints_mut(&mut self) -> &mut CheckpointManager {
        &mut self.checkpoints
    }

    /// Set tab width
    pub fn set_tab_width(&mut self, width: usize) {
        self.tab_width = width;
    }

    /// Toggle line numbers
    pub fn toggle_line_numbers(&mut self) {
        self.show_line_numbers = !self.show_line_numbers;
    }

    /// Check if line numbers are shown
    pub fn shows_line_numbers(&self) -> bool {
        self.show_line_numbers
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}
