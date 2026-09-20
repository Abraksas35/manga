//! UI module - User interface components and rendering

use vello::peniko::Color;
use crate::editor::Editor;
use crate::time_tracker::TimeTracker;
use crate::music_player::{MusicPlayer, PlayerState, PlaybackMode};
use crate::wallpaper::WallpaperManager;

/// UI state for the IDE
pub struct UI {
    /// Whether the music player panel is expanded
    pub music_panel_expanded: bool,
    /// Whether the history timeline is visible
    pub history_timeline_visible: bool,
    /// History timeline slider position (0.0 to 1.0)
    pub history_slider_position: f32,
    /// Whether settings panel is visible
    pub settings_visible: bool,
    /// Current theme
    pub theme: Theme,
}

/// Color theme
#[derive(Debug, Clone)]
pub struct Theme {
    pub background: Color,
    pub foreground: Color,
    pub line_number_bg: Color,
    pub line_number_fg: Color,
    pub selection: Color,
    pub cursor: Color,
    pub status_bar_bg: Color,
    pub status_bar_fg: Color,
    pub panel_bg: Color,
    pub panel_fg: Color,
    pub button_bg: Color,
    pub button_fg: Color,
    pub button_hover_bg: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: Color::rgb8(30, 30, 30),
            foreground: Color::rgb8(212, 212, 212),
            line_number_bg: Color::rgb8(40, 40, 40),
            line_number_fg: Color::rgb8(100, 100, 100),
            selection: Color::rgba8(50, 100, 200, 100),
            cursor: Color::rgb8(200, 200, 200),
            status_bar_bg: Color::rgb8(20, 20, 20),
            status_bar_fg: Color::rgb8(180, 180, 180),
            panel_bg: Color::rgba8(20, 20, 20, 240),
            panel_fg: Color::rgb8(200, 200, 200),
            button_bg: Color::rgb8(50, 50, 50),
            button_fg: Color::rgb8(220, 220, 220),
            button_hover_bg: Color::rgb8(70, 70, 70),
        }
    }
}

impl UI {
    /// Create a new UI
    pub fn new() -> Self {
        Self {
            music_panel_expanded: false,
            history_timeline_visible: false,
            history_slider_position: 1.0,
            settings_visible: false,
            theme: Theme::default(),
        }
    }

    /// Render the entire UI
    pub fn render(
        &self,
        editor: &Editor,
        time_tracker: &TimeTracker,
        music_player: &MusicPlayer,
        _wallpaper: &WallpaperManager,
        // render_context: &mut impl VelloRenderContext,
    ) {
        // Background/wallpaper would be rendered first
        
        // Editor content with syntax highlighting
        self.render_editor(editor);
        
        // Status bar
        self.render_status_bar(editor, time_tracker);
        
        // Music player panel (if expanded)
        if self.music_panel_expanded {
            self.render_music_panel(music_player);
        }
        
        // History timeline (if visible)
        if self.history_timeline_visible {
            self.render_history_timeline(editor);
        }
        
        // Settings panel (if visible)
        if self.settings_visible {
            self.render_settings_panel();
        }
    }

    /// Render the editor area
    fn render_editor(&self, editor: &Editor) {
        // This would use Vello to render:
        // - Line numbers
        // - Text with syntax highlighting
        // - Cursor
        // - Selection
        
        let buffer = editor.buffer();
        let _highlights = editor.get_highlights();
        let viewport = editor.viewport();
        
        // Render each visible line
        for line_idx in viewport.scroll_y..(viewport.scroll_y + viewport.visible_lines).min(buffer.len_lines()) {
            if let Some(line_content) = buffer.line(line_idx) {
                // Render line number
                self.render_line_number(line_idx + 1);
                
                // Render line content with highlights
                self.render_line_content(&line_content, line_idx, &[]);
            }
        }
        
        // Render cursor
        let (cursor_line, cursor_col) = buffer.cursor();
        self.render_cursor(cursor_line, cursor_col, viewport);
    }

    /// Render a line number
    fn render_line_number(&self, _line: usize) {
        // Would render the line number in the gutter area
        // using Vello's text rendering
    }

    /// Render line content with syntax highlighting
    fn render_line_content(&self, _content: &str, _line_idx: usize, _highlights: &[crate::syntax_highlighter::HighlightedSegment]) {
        // Would render the text with appropriate colors based on highlights
    }

    /// Render the cursor
    fn render_cursor(&self, _line: usize, _col: usize, _viewport: &crate::editor::Viewport) {
        // Would render a blinking cursor at the current position
    }

    /// Render the status bar
    fn render_status_bar(&self, editor: &Editor, time_tracker: &TimeTracker) {
        let buffer = editor.buffer();
        
        // File name or "Untitled"
        let _file_name = buffer.file_name().unwrap_or("Untitled");
        
        // Cursor position
        let (_line, _col) = buffer.cursor();
        
        // Time spent
        let _time_str = if let Some(_path) = buffer.path() {
            if let Some(seconds) = time_tracker.get_current_file_time() {
                TimeTracker::format_duration(seconds)
            } else {
                "0s".to_string()
            }
        } else {
            "0s".to_string()
        };
        
        // Dirty indicator
        let _dirty_indicator = if buffer.is_dirty() { "●" } else { "" };
        
        // Status bar content:
        // [file_name] [dirty] | Ln X, Col Y | Time: Z | Encoding: UTF-8 | Language: Rust
    }

    /// Render the music player panel
    fn render_music_panel(&self, music_player: &MusicPlayer) {
        // Collapsed state: small bar showing current track
        // Expanded state: full panel with:
        // - Track title
        // - Artist
        // - Progress bar
        // - Play/Pause button
        // - Next/Previous buttons
        // - Shuffle mode indicator
        // - Volume control
        
        if let Some(_track) = music_player.current_track() {
            let _state_icon = match music_player.state() {
                PlayerState::Playing => "▶",
                PlayerState::Paused => "⏸",
                PlayerState::Stopped => "⏹",
            };
            
            let _shuffle_icon = if music_player.playback_mode() == PlaybackMode::Shuffle {
                "🔀"
            } else {
                ""
            };
        }
    }

    /// Render the history timeline
    fn render_history_timeline(&self, _editor: &Editor) {
        // Timeline slider at the bottom
        // Shows code evolution over time
        // Can scrub through to see old versions
        
        // Would show:
        // - Timeline with markers for saves/checkpoints
        // - Slider to navigate through time
        // - Preview of code at selected time
    }

    /// Render the settings panel
    fn render_settings_panel(&self) {
        // Settings options:
        // - Theme selection
        // - Font size
        // - Tab width
        // - Line numbers toggle
        // - Wallpaper selection
        // - Music directory
    }

    /// Handle mouse click
    pub fn handle_mouse_click(&mut self, _x: f64, _y: f64, _editor: &mut Editor) -> bool {
        // Check if clicking on music player area
        // Check if clicking on timeline
        // Check if clicking on editor content
        
        false
    }

    /// Handle mouse scroll
    pub fn handle_mouse_scroll(&mut self, delta_x: f64, delta_y: f64, modifiers: winit::keyboard::ModifiersState, editor: &mut Editor) -> bool {
        // Ctrl+Scroll for horizontal scrolling
        if modifiers.contains(winit::keyboard::ModifiersState::CONTROL) {
            editor.scroll_horizontal(delta_x as i32);
            return true;
        }
        
        // Normal vertical scroll
        let viewport = editor.viewport_mut();
        viewport.scroll_y = viewport.scroll_y.saturating_add(delta_y as usize);
        
        true
    }

    /// Toggle music panel
    pub fn toggle_music_panel(&mut self) {
        self.music_panel_expanded = !self.music_panel_expanded;
    }

    /// Toggle history timeline
    pub fn toggle_history_timeline(&mut self) {
        self.history_timeline_visible = !self.history_timeline_visible;
    }

    /// Toggle settings panel
    pub fn toggle_settings(&mut self) {
        self.settings_visible = !self.settings_visible;
    }

    /// Update history slider position
    pub fn set_history_slider(&mut self, position: f32) {
        self.history_slider_position = position.clamp(0.0, 1.0);
    }

    /// Get the code at the current history position
    pub fn get_historical_code(&self, editor: &Editor) -> Option<String> {
        if !self.history_timeline_visible {
            return None;
        }
        
        // Get the current buffer path
        let _path = editor.buffer().path()?;
        
        // Calculate the time based on slider position
        // This is simplified - would need actual time range from history
        let _history = editor.history();
        
        // Get snapshots and find the one at the slider position
        None // Placeholder
    }
}

impl Default for UI {
    fn default() -> Self {
        Self::new()
    }
}
