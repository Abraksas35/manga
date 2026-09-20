//! Wallpaper module - Supports static and animated wallpapers

use std::path::PathBuf;
use std::time::Duration;
use image::DynamicImage;

/// Type of wallpaper
#[derive(Debug, Clone, PartialEq)]
pub enum WallpaperType {
    /// Static image (PNG, JPG, etc.)
    Static,
    /// Animated GIF
    AnimatedGif,
    /// Custom animation (frames provided)
    CustomAnimation,
}

// Manual Serialize implementation for WallpaperType
impl serde::Serialize for WallpaperType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("WallpaperType", 1)?;
        match self {
            WallpaperType::Static => state.serialize_field("type", "static")?,
            WallpaperType::AnimatedGif => state.serialize_field("type", "animated_gif")?,
            WallpaperType::CustomAnimation => state.serialize_field("type", "custom_animation")?,
        }
        state.end()
    }
}

// Manual Deserialize implementation for WallpaperType
impl<'de> serde::Deserialize<'de> for WallpaperType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct WallpaperTypeHelper {
            #[serde(rename = "type")]
            type_field: String,
        }

        let helper = WallpaperTypeHelper::deserialize(deserializer)?;
        match helper.type_field.as_str() {
            "static" => Ok(WallpaperType::Static),
            "animated_gif" => Ok(WallpaperType::AnimatedGif),
            "custom_animation" => Ok(WallpaperType::CustomAnimation),
            _ => Err(serde::de::Error::unknown_variant(
                &helper.type_field,
                &["static", "animated_gif", "custom_animation"],
            )),
        }
    }
}

/// Wallpaper data - not serializable due to DynamicImage and Instant
#[derive(Debug, Clone)]
pub struct Wallpaper {
    /// Path to the wallpaper file
    pub path: PathBuf,
    /// Type of wallpaper
    pub wallpaper_type: WallpaperType,
    /// Animation frames (for animated wallpapers) - skipped during serialization
    pub frames: Vec<DynamicImage>,
    /// Current frame index
    pub current_frame: usize,
    /// Frame duration for animations
    pub frame_duration: Duration,
    /// Last frame change time - skipped during serialization
    pub last_frame_change: std::time::Instant,
    /// Whether the wallpaper is loaded
    pub is_loaded: bool,
    /// Optional name/description
    pub name: Option<String>,
}

// Manual Serialize implementation to skip non-serializable fields
impl serde::Serialize for Wallpaper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Wallpaper", 6)?;
        state.serialize_field("path", &self.path)?;
        state.serialize_field("wallpaper_type", &self.wallpaper_type)?;
        // Skip frames
        state.serialize_field("current_frame", &self.current_frame)?;
        state.serialize_field("frame_duration", &self.frame_duration)?;
        // Skip last_frame_change
        state.serialize_field("is_loaded", &self.is_loaded)?;
        state.serialize_field("name", &self.name)?;
        state.end()
    }
}

// Manual Deserialize implementation to set default values for skipped fields
impl<'de> serde::Deserialize<'de> for Wallpaper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct WallpaperHelper {
            path: PathBuf,
            wallpaper_type: WallpaperType,
            current_frame: usize,
            frame_duration: Duration,
            is_loaded: bool,
            name: Option<String>,
        }

        let helper = WallpaperHelper::deserialize(deserializer)?;
        Ok(Wallpaper {
            path: helper.path,
            wallpaper_type: helper.wallpaper_type,
            frames: Vec::new(),
            current_frame: helper.current_frame,
            frame_duration: helper.frame_duration,
            last_frame_change: std::time::Instant::now(),
            is_loaded: helper.is_loaded,
            name: helper.name,
        })
    }
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

        if let Ok(Some(frame)) = reader.next_frame_info() {
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
        self.frames.first().map(|f| (f.width(), f.height()))
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

        if let Ok(entries) = std::fs::read_dir(&self.wallpapers_dir) {
            for entry in entries.flatten() {
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
