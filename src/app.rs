//! App module - Main application state and event loop

use winit::{
    application::ApplicationHandler,
    event::{WindowEvent, ElementState, KeyEvent, MouseScrollDelta},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, ModifiersState, PhysicalKey},
    window::{Window, WindowId, WindowAttributes},
};
use log::{info, error};

use crate::editor::Editor;
use crate::time_tracker::TimeTracker;
use crate::wallpaper::WallpaperManager;
use crate::music_player::MusicPlayer;
use crate::ui::UI;
use crate::config::Config;

/// Main application struct
pub struct AuroraApp {
    /// The main window
    window: Option<std::sync::Arc<Window>>,
    /// Editor instance
    editor: Editor,
    /// Time tracker
    time_tracker: TimeTracker,
    /// Wallpaper manager
    wallpaper: WallpaperManager,
    /// Music player
    #[allow(dead_code)]
    music_player: MusicPlayer,
    /// UI state
    ui: UI,
    /// Configuration
    config: Config,
    /// Keyboard modifiers
    modifiers: ModifiersState,
    /// Whether the app should close
    should_close: bool,
}

impl AuroraApp {
    /// Create a new application
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Load configuration
        let config = Config::load().unwrap_or_default();
        
        Ok(Self {
            window: None,
            editor: Editor::new(),
            time_tracker: TimeTracker::new(),
            wallpaper: WallpaperManager::new(),
            music_player: MusicPlayer::new(),
            ui: UI::new(),
            config,
            modifiers: ModifiersState::empty(),
            should_close: false,
        })
    }

    /// Run the application
    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        info!("Initializing Aurora IDE...");
        
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Wait);
        
        let mut app = Self::new()?;
        
        // Load persisted data
        let _ = app.time_tracker.load();
        let _ = app.editor.history_mut().load();
        let _ = app.editor.checkpoints_mut().load();
        
        info!("Starting event loop...");
        event_loop.run_app(&mut app)?;
        
        // Save data on exit
        let _ = app.time_tracker.save();
        let _ = app.editor.history_mut().save();
        let _ = app.editor.checkpoints_mut().save();
        let _ = app.config.save();
        
        Ok(())
    }

    /// Create the main window
    fn create_window(&mut self, event_loop: &ActiveEventLoop) -> Result<std::sync::Arc<Window>, Box<dyn std::error::Error>> {
        let attrs = WindowAttributes::default()
            .with_title("Aurora IDE")
            .with_inner_size(winit::dpi::LogicalSize::new(
                self.config.ui.window_width,
                self.config.ui.window_height,
            ));
        
        let window = event_loop.create_window(attrs)?;
        let window = std::sync::Arc::new(window);
        
        Ok(window)
    }

    /// Handle keyboard input
    fn handle_keyboard_input(&mut self, key: KeyEvent, state: ElementState) {
        if state != ElementState::Pressed {
            return;
        }

        match key.physical_key {
            PhysicalKey::Code(key_code) => {
                // Check for special key combinations
                if self.handle_shortcut(key_code) {
                    return;
                }

                // Pass to editor
                self.editor.handle_key(key_code, self.modifiers);
            }
            _ => {}
        }
    }

    /// Handle keyboard shortcuts
    fn handle_shortcut(&mut self, key: KeyCode) -> bool {
        let mods = &self.modifiers;
        
        match key {
            // Ctrl+Q or Alt+F4 - Quit
            KeyCode::KeyQ if mods.contains(ModifiersState::CONTROL) => {
                self.should_close = true;
                return true;
            }
            
            // Ctrl+O - Open file
            KeyCode::KeyO if mods.contains(ModifiersState::CONTROL) => {
                self.open_file_dialog();
                return true;
            }
            
            // Ctrl+N - New file
            KeyCode::KeyN if mods.contains(ModifiersState::CONTROL) => {
                self.editor.new_file();
                return true;
            }
            
            // Ctrl+S is handled by editor
            
            // Alt+Shift+S - Checkpoint save (handled by editor)
            
            // Ctrl+M - Toggle music panel
            KeyCode::KeyM if mods.contains(ModifiersState::CONTROL) => {
                self.ui.toggle_music_panel();
                return true;
            }
            
            // Ctrl+T - Toggle history timeline
            KeyCode::KeyT if mods.contains(ModifiersState::CONTROL) => {
                self.ui.toggle_history_timeline();
                return true;
            }
            
            // Ctrl+, - Toggle settings
            KeyCode::Comma if mods.contains(ModifiersState::CONTROL) => {
                self.ui.toggle_settings();
                return true;
            }
            
            // F11 - Toggle fullscreen
            KeyCode::F11 => {
                if let Some(window) = &self.window {
                    let is_fullscreen = window.fullscreen().is_some();
                    if is_fullscreen {
                        window.set_fullscreen(None);
                    } else {
                        window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
                    }
                }
                return true;
            }
            
            // F5 - Run/Build (placeholder)
            KeyCode::F5 => {
                info!("Build/Run triggered (not implemented)");
                return true;
            }
            
            _ => false,
        }
    }

    /// Open file dialog (placeholder - would use rfd crate)
    fn open_file_dialog(&mut self) {
        info!("Open file dialog (would show file picker)");
        // In a real implementation, we'd use the rfd crate:
        // if let Some(path) = rfd::FileDialog::new().pick_file() {
        //     let _ = self.editor.open_file(&path);
        //     self.time_tracker.start_file(path);
        // }
    }

    /// Handle mouse scroll
    fn handle_mouse_scroll(&mut self, delta: MouseScrollDelta) {
        let (delta_x, delta_y) = match delta {
            MouseScrollDelta::LineDelta(x, y) => (x as f64 * 20.0, y as f64 * 20.0),
            MouseScrollDelta::PixelDelta(d) => (d.x, d.y),
        };

        self.ui.handle_mouse_scroll(delta_x, delta_y, self.modifiers, &mut self.editor);
    }

    /// Update animation frame
    fn update(&mut self) {
        // Update wallpaper animation
        if self.wallpaper.is_enabled() {
            self.wallpaper.update();
        }
        
        // Update music player position (placeholder)
        // In a real implementation, we'd track actual playback time
        
        // Auto-save if enabled
        // This would be tracked with a timer in a real implementation
    }

    /// Render the UI
    fn render(&mut self) {
        // In a real implementation, this would:
        // 1. Get the window surface
        // 2. Create a Vello render context
        // 3. Render wallpaper background
        // 4. Render editor content with syntax highlighting
        // 5. Render UI panels (status bar, music player, timeline)
        // 6. Present the frame
        
        // Placeholder - just log that we're rendering
        // In production, this would use Vello's async renderer
    }
}

impl ApplicationHandler for AuroraApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            match self.create_window(event_loop) {
                Ok(window) => {
                    info!("Window created successfully");
                    self.window = Some(window);
                }
                Err(e) => {
                    error!("Failed to create window: {}", e);
                }
            }
        }
    }

    fn window_event(&mut self, _event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                info!("Close requested");
                self.should_close = true;
            }
            
            WindowEvent::KeyboardInput { ref event, .. } => {
                if event.state == ElementState::Pressed {
                    // Modifiers are handled separately via ModifiersChanged
                }
                self.handle_keyboard_input(event.clone(), event.state);
            }
            
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers.state();
            }
            
            WindowEvent::MouseWheel { delta, .. } => {
                self.handle_mouse_scroll(delta);
            }
            
            WindowEvent::MouseInput { state: _, button: _, .. } => {
                // Handle mouse clicks for UI interaction
            }
            
            WindowEvent::CursorMoved { position: _, .. } => {
                // Track cursor position for hover effects
            }
            
            WindowEvent::RedrawRequested => {
                self.update();
                self.render();
                
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            
            WindowEvent::Resized(size) => {
                // Update viewport based on new size
                let visible_lines = (size.height as f32 / 20.0) as usize; // Assuming ~20px per line
                let visible_columns = (size.width as f32 / 10.0) as usize; // Assuming ~10px per character
                self.editor.set_viewport_size(visible_lines, visible_columns);
            }
            
            WindowEvent::Focused(focused) => {
                if focused {
                    // Resume time tracking when window regains focus
                    if let Some(path) = self.editor.buffer().path() {
                        self.time_tracker.start_file(path.clone());
                    }
                } else {
                    // Pause time tracking when window loses focus
                    self.time_tracker.end_current_session();
                }
            }
            
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.should_close {
            event_loop.exit();
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        info!("Exiting Aurora IDE...");
        
        // Save all data
        let _ = self.time_tracker.save();
        let _ = self.editor.history_mut().save();
        let _ = self.editor.checkpoints_mut().save();
        let _ = self.config.save();
        
        // End current time tracking session
        self.time_tracker.end_current_session();
    }
}

impl Default for AuroraApp {
    fn default() -> Self {
        Self::new().expect("Failed to create AuroraApp")
    }
}
