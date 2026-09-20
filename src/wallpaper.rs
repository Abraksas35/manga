//! Wallpaper module - Supports static and animated wallpapers

use std::path::PathBuf;
use std::time::Duration;
use image::{DynamicImage, ImageFormat};
use serde::{Serialize, Deserialize};

/// Type of wallpaper
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WallpaperType {
    /// Static image (PNG, JPG, etc.)
    Static,
    /// Animated GIF
    AnimatedGif,
    /// Custom animation (frames provided)
    CustomAnimation,
}

/// Wallpaper data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallpaper {
    /// Path to the wallpaper file
    pub path: PathBuf,
    /// Type of wallpaper
    pub wallpaper_type: WallpaperType,
    /// Animation frames (for animated wallpapers)
    pub frames: Vec<DynamicImage>,
    /// Current frame index
    pub current_frame: usize,
    /// Frame duration for animations
    pub frame_duration: Duration,
    /// Last frame change time
    pub last_frame_change: std::time::Instant,
    /// Whether the wallpaper is loaded
    pub is_loaded: bool,
    /// Optional name/description
    pub name: Option<String>,
}

impl Wallpaper {
    /// Create a new wallpaper from a file path
    pub fn from_path(path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let wallpaper_type = match extension.as_str() {
            "gif" => WallpaperType::AnimatedGif,
            "png" | "jpg" | "jpeg" | "bmp" | "webp" => WallpaperType::Static,
            _ => WallpaperType::Static,
        };

        let mut wallpaper = Self {
            path: path.clone(),
            wallpaper_type,
            frames: Vec::new(),
            current_frame: 0,
            frame_duration: Duration::from_millis(100),
            last_frame_change: std::time::Instant::now(),
            is_loaded: false,
            name: path.file_stem().and_then(|s| s.to_str()).map(String::from),
        };

        // Load the wallpaper
        wallpaper.load()?;

        Ok(wallpaper)
    }

    /// Load the wallpaper from disk
    pub fn load(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self.wallpaper_type {
            WallpaperType::AnimatedGif => {
                self.load_gif()?;
            }
            WallpaperType::Static => {
                self.load_static()?;
            }
            WallpaperType::CustomAnimation => {
                // Frames should be provided manually
            }
        }

        self.is_loaded = true;
        Ok(())
    }

    /// Load a static image
    fn load_static(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let img = image::open(&self.path)?;
        self.frames = vec![img];
        Ok(())
    }

    /// Load an animated GIF
    fn load_gif(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let file = std::fs::File::open(&self.path)?;
        let decoder = gif::DecodeOptions::new();
        let mut reader = decoder.read_info(file)?;

        self.frames.clear();

        if let Some(frame) = reader.next_frame_info() {
            // Set frame duration from first frame
            self.frame_duration = Duration::from_millis(frame.delay as u64 * 10);
        }

        while let Ok(Some(frame)) = reader.read_next_frame() {
            let img = DynamicImage::ImageRgba8(
                image::ImageBuffer::from_raw(
                    frame.width as u32,
                    frame.height as u32,
                    frame.buffer.to_vec(),
                ).unwrap()
            );
            self.frames.push(img);
        }

        if self.frames.is_empty() {
            // Fallback to static load if no frames found
            self.load_static()?;
            self.wallpaper_type = WallpaperType::Static;
        }

        Ok(())
    }

    /// Get the current frame
    pub fn current_frame(&self) -> Option<&DynamicImage> {
        self.frames.get(self.current_frame)
    }

    /// Update animation frame if needed
    pub fn update(&mut self) {
        if self.wallpaper_type == WallpaperType::Static || self.frames.len() <= 1 {
            return;
        }

        if self.last_frame_change.elapsed() >= self.frame_duration {
            self.current_frame = (self.current_frame + 1) % self.frames.len();
            self.last_frame_change = std::time::Instant::now();
        }
    }

    /// Check if wallpaper is animated
    pub fn is_animated(&self) -> bool {
        self.wallpaper_type != WallpaperType::Static && self.frames.len() > 1
    }

    /// Get wallpaper dimensions
    pub fn dimensions(&self) -> Option<(u32, u32)> {
        self.frames.first().map(|f| f.dimensions())
    }

    /// Set custom frames for animation
    pub fn set_custom_frames(&mut self, frames: Vec<DynamicImage>, frame_duration: Duration) {
        self.frames = frames;
        self.frame_duration = frame_duration;
        self.wallpaper_type = WallpaperType::CustomAnimation;
        self.is_loaded = !self.frames.is_empty();
    }

    /// Reset to first frame
    pub fn reset(&mut self) {
        self.current_frame = 0;
        self.last_frame_change = std::time::Instant::now();
    }
}

/// Wallpaper manager for the IDE
pub struct WallpaperManager {
    /// Current active wallpaper
    current_wallpaper: Option<Wallpaper>,
    /// List of available wallpapers
    available_wallpapers: Vec<PathBuf>,
    /// Wallpapers directory
    wallpapers_dir: PathBuf,
    /// Whether wallpaper is enabled
    enabled: bool,
}

impl WallpaperManager {
    /// Create a new wallpaper manager
    pub fn new() -> Self {
        let wallpapers_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".aurora_ide")
            .join("wallpapers");

        let mut manager = Self {
            current_wallpaper: None,
            available_wallpapers: Vec::new(),
            wallpapers_dir,
            enabled: true,
        };

        // Scan for wallpapers
        manager.scan_wallpapers();

        manager
    }

    /// Scan the wallpapers directory for available wallpapers
    pub fn scan_wallpapers(&mut self) {
        self.available_wallpapers.clear();

        if !self.wallpapers_dir.exists() {
            let _ = std::fs::create_dir_all(&self.wallpapers_dir);
            return;
        }

        for entry in std::fs::read_dir(&self.wallpapers_dir).unwrap_or_else(|_| std::fs::ReadDir::empty()) {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if ["png", "jpg", "jpeg", "gif", "bmp", "webp"].contains(&ext.to_lowercase().as_str()) {
                            self.available_wallpapers.push(path);
                        }
                    }
                }
            }
        }
    }

    /// Set the current wallpaper by path
    pub fn set_wallpaper(&mut self, path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let wallpaper = Wallpaper::from_path(path)?;
        self.current_wallpaper = Some(wallpaper);
        Ok(())
    }

    /// Set wallpaper by index from available wallpapers
    pub fn set_wallpaper_by_index(&mut self, index: usize) -> Result<(), Box<dyn std::error::Error>> {
        if index >= self.available_wallpapers.len() {
            return Err("Invalid wallpaper index".into());
        }

        let path = self.available_wallpapers[index].clone();
        self.set_wallpaper(path)
    }

    /// Get the current wallpaper
    pub fn current_wallpaper(&self) -> Option<&Wallpaper> {
        self.current_wallpaper.as_ref()
    }

    /// Get mutable reference to current wallpaper
    pub fn current_wallpaper_mut(&mut self) -> Option<&mut Wallpaper> {
        self.current_wallpaper.as_mut()
    }

    /// Update wallpaper animation
    pub fn update(&mut self) {
        if let Some(wallpaper) = &mut self.current_wallpaper {
            wallpaper.update();
        }
    }

    /// Enable/disable wallpaper
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if wallpaper is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get list of available wallpapers
    pub fn get_available_wallpapers(&self) -> &[PathBuf] {
        &self.available_wallpapers
    }

    /// Remove current wallpaper
    pub fn clear_wallpaper(&mut self) {
        self.current_wallpaper = None;
    }

    /// Cycle to next wallpaper
    pub fn next_wallpaper(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.available_wallpapers.is_empty() {
            return Err("No wallpapers available".into());
        }

        let current_index = self.current_wallpaper.as_ref()
            .and_then(|w| self.available_wallpapers.iter().position(|p| p == &w.path))
            .unwrap_or(self.available_wallpapers.len().saturating_sub(1));

        let next_index = (current_index + 1) % self.available_wallpapers.len();
        self.set_wallpaper_by_index(next_index)
    }

    /// Cycle to previous wallpaper
    pub fn prev_wallpaper(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.available_wallpapers.is_empty() {
            return Err("No wallpapers available".into());
        }

        let current_index = self.current_wallpaper.as_ref()
            .and_then(|w| self.available_wallpapers.iter().position(|p| p == &w.path))
            .unwrap_or(0);

        let prev_index = if current_index == 0 {
            self.available_wallpapers.len() - 1
        } else {
            current_index - 1
        };

        self.set_wallpaper_by_index(prev_index)
    }
}

impl Default for WallpaperManager {
    fn default() -> Self {
        Self::new()
    }
}
