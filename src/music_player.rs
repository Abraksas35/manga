//! Music player module - Built-in music player with shuffle mode

use std::path::PathBuf;
use std::collections::VecDeque;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Serialize, Deserialize};
use rodio::{Source, OutputStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use log::warn;

/// Track information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    /// Path to the audio file
    pub path: PathBuf,
    /// Track title (extracted from filename if no metadata)
    pub title: String,
    /// Artist (extracted from filename or folder structure)
    pub artist: Option<String>,
    /// Duration of the track
    pub duration: Option<Duration>,
}

impl Track {
    /// Create a track from a file path
    pub fn from_path(path: PathBuf) -> Self {
        let title = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        Self {
            path,
            title,
            artist: None,
            duration: None,
        }
    }
}

/// Playback mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackMode {
    /// Sequential playback
    Sequential,
    /// Shuffle/random playback
    Shuffle,
    /// Repeat single track
    RepeatOne,
    /// Repeat all
    RepeatAll,
}

/// Player state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerState {
    Stopped,
    Playing,
    Paused,
}

/// Music player
pub struct MusicPlayer {
    /// List of all tracks in the music directory
    all_tracks: Vec<Track>,
    /// Current playlist (may be shuffled)
    playlist: VecDeque<usize>,
    /// Current track index in all_tracks
    current_track_index: Option<usize>,
    /// Player state
    state: PlayerState,
    /// Playback mode
    playback_mode: PlaybackMode,
    /// Current playback position
    current_position: Duration,
    /// Audio output stream
    _stream: Option<OutputStream>,
    /// Audio sink
    sink: Option<Arc<Mutex<rodio::Sink>>>,
    /// Music directory path
    music_dir: PathBuf,
    /// Volume (0.0 to 1.0)
    volume: f32,
}

impl MusicPlayer {
    /// Create a new music player
    pub fn new() -> Self {
        let music_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Music")
            .join("aurora_ide");

        let mut player = Self {
            all_tracks: Vec::new(),
            playlist: VecDeque::new(),
            current_track_index: None,
            state: PlayerState::Stopped,
            playback_mode: PlaybackMode::Shuffle, // Default to shuffle as requested
            current_position: Duration::ZERO,
            _stream: None,
            sink: None,
            music_dir,
            volume: 0.7,
        };

        // Scan for tracks
        player.scan_tracks();

        // Initialize audio output
        player.init_audio();

        player
    }

    /// Initialize audio output
    fn init_audio(&mut self) {
        match OutputStream::try_default() {
            Ok((stream, handle)) => {
                self._stream = Some(stream);
                // Sink will be created when playing
            }
            Err(e) => {
                log::warn!("Failed to initialize audio output: {}", e);
            }
        }
    }

    /// Scan the music directory for tracks
    pub fn scan_tracks(&mut self) {
        self.all_tracks.clear();

        if !self.music_dir.exists() {
            let _ = std::fs::create_dir_all(&self.music_dir);
            return;
        }

        self.scan_directory(&self.music_dir);
        
        // Sort tracks by title
        self.all_tracks.sort_by(|a, b| a.title.cmp(&b.title));
    }

    /// Recursively scan a directory for audio files
    fn scan_directory(&mut self, dir: &PathBuf) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                if path.is_dir() {
                    self.scan_directory(&path);
                } else if path.is_file() {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if ["mp3", "wav", "ogg", "flac", "m4a", "aac"].contains(&ext.to_lowercase().as_str()) {
                            let mut track = Track::from_path(path.clone());
                            
                            // Try to extract artist from parent folder name
                            if let Some(parent) = path.parent() {
                                if let Some(name) = parent.file_name().and_then(|n| n.to_str()) {
                                    if name != self.music_dir.file_name().and_then(|n| n.to_str()).unwrap_or("") {
                                        track.artist = Some(name.to_string());
                                    }
                                }
                            }
                            
                            self.all_tracks.push(track);
                        }
                    }
                }
            }
        }
    }

    /// Build the playlist based on playback mode
    fn build_playlist(&mut self) {
        self.playlist.clear();
        
        let indices: Vec<usize> = (0..self.all_tracks.len()).collect();
        
        match self.playback_mode {
            PlaybackMode::Shuffle => {
                let mut shuffled = indices;
                shuffled.shuffle(&mut thread_rng());
                self.playlist.extend(shuffled);
            }
            _ => {
                self.playlist.extend(indices);
            }
        }
    }

    /// Play the current track
    pub fn play(&mut self) {
        if self.all_tracks.is_empty() || self._stream.is_none() {
            return;
        }

        if let Some(index) = self.current_track_index {
            if let Some(track) = self.all_tracks.get(index) {
                if let Ok(file) = std::fs::File::open(&track.path) {
                    match rodio::Decoder::new(file) {
                        Ok(source) => {
                            // Create a new sink for playback
                            if let Ok(sink) = rodio::Sink::try_new(&self._stream.as_ref().unwrap()) {
                                sink.append(source);
                                self.sink = Some(Arc::new(Mutex::new(sink)));
                                self.state = PlayerState::Playing;
                            }
                        }
                        Err(e) => warn!("Failed to decode audio file: {}", e),
                    }
                }
            }
        }
    }

    /// Play a specific track by index
    pub fn play_track(&mut self, index: usize) {
        if index >= self.all_tracks.len() {
            return;
        }

        self.current_track_index = Some(index);
        self.current_position = Duration::ZERO;
        self.play();
    }

    /// Pause playback
    pub fn pause(&mut self) {
        if let Some(ref sink) = self.sink {
            // Note: rodio doesn't have built-in pause, so we stop for now
            // A more advanced implementation would use a different approach
        }
        self.state = PlayerState::Paused;
    }

    /// Stop playback
    pub fn stop(&mut self) {
        self.sink = None;
        self.state = PlayerState::Stopped;
        self.current_position = Duration::ZERO;
    }

    /// Play next track
    pub fn next(&mut self) {
        if self.playlist.is_empty() {
            self.build_playlist();
        }

        if let Some(current) = self.current_track_index {
            // Find current position in playlist
            if let Some(pos) = self.playlist.iter().position(|&i| i == current) {
                if pos + 1 < self.playlist.len() {
                    let next_index = self.playlist[pos + 1];
                    self.play_track(next_index);
                    return;
                }
            }
        }

        // If we're at the end, handle based on mode
        match self.playback_mode {
            PlaybackMode::RepeatAll | PlaybackMode::Shuffle => {
                self.build_playlist();
                if let Some(&first) = self.playlist.front() {
                    self.play_track(first);
                }
            }
            _ => {
                self.stop();
            }
        }
    }

    /// Play previous track
    pub fn prev(&mut self) {
        if self.playlist.is_empty() {
            self.build_playlist();
        }

        if let Some(current) = self.current_track_index {
            if let Some(pos) = self.playlist.iter().position(|&i| i == current) {
                if pos > 0 {
                    let prev_index = self.playlist[pos - 1];
                    self.play_track(prev_index);
                    return;
                }
            }
        }

        // If we're at the beginning, go to the last track
        if let Some(&last) = self.playlist.back() {
            self.play_track(last);
        }
    }

    /// Toggle shuffle mode
    pub fn toggle_shuffle(&mut self) {
        self.playback_mode = match self.playback_mode {
            PlaybackMode::Shuffle => PlaybackMode::Sequential,
            _ => PlaybackMode::Shuffle,
        };
        
        // Rebuild playlist if we're changing to/from shuffle
        if self.playback_mode == PlaybackMode::Shuffle {
            self.build_playlist();
        }
    }

    /// Cycle through playback modes
    pub fn cycle_playback_mode(&mut self) {
        self.playback_mode = match self.playback_mode {
            PlaybackMode::Sequential => PlaybackMode::Shuffle,
            PlaybackMode::Shuffle => PlaybackMode::RepeatOne,
            PlaybackMode::RepeatOne => PlaybackMode::RepeatAll,
            PlaybackMode::RepeatAll => PlaybackMode::Sequential,
        };
    }

    /// Get current track
    pub fn current_track(&self) -> Option<&Track> {
        self.current_track_index.and_then(|i| self.all_tracks.get(i))
    }

    /// Get player state
    pub fn state(&self) -> PlayerState {
        self.state
    }

    /// Get playback mode
    pub fn playback_mode(&self) -> PlaybackMode {
        self.playback_mode
    }

    /// Get current position
    pub fn current_position(&self) -> Duration {
        self.current_position
    }

    /// Get total tracks
    pub fn total_tracks(&self) -> usize {
        self.all_tracks.len()
    }

    /// Get all tracks
    pub fn all_tracks(&self) -> &[Track] {
        &self.all_tracks
    }

    /// Set volume
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
        // Apply volume to sink if available
    }

    /// Get volume
    pub fn volume(&self) -> f32 {
        self.volume
    }

    /// Check if there are any tracks
    pub fn has_tracks(&self) -> bool {
        !self.all_tracks.is_empty()
    }

    /// Get music directory path
    pub fn music_dir(&self) -> &PathBuf {
        &self.music_dir
    }
}

impl Default for MusicPlayer {
    fn default() -> Self {
        Self::new()
    }
}
