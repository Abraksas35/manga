//! Aurora IDE - A modern, GPU-accelerated code editor
//! 
//! Features:
//! - Time tracking per file/project
//! - Code history timeline with slider
//! - Ctrl+Scroll for horizontal scrolling
//! - Checkpoint saves (Alt+Shift+S) with 10 last checkpoints
//! - Animated/static wallpapers
//! - Built-in music player with shuffle mode
//! - Syntax highlighting via tree-sitter
//! - GPU rendering with wgpu/vello

mod app;
mod editor;
mod time_tracker;
mod history;
mod checkpoint;
mod wallpaper;
mod music_player;
mod syntax_highlighter;
mod buffer;
mod ui;
mod config;

use app::AuroraApp;
use log::{info, error};

fn main() {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    info!("Starting Aurora IDE...");
    
    // Run the application
    match AuroraApp::run() {
        Ok(_) => info!("Aurora IDE closed successfully"),
        Err(e) => error!("Aurora IDE crashed: {}", e),
    }
}
