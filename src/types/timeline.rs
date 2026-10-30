use crate::types::effects::{BlendMode, ItemEffect};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrackKind {
    Video,
    Audio,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub name: String,
    pub kind: TrackKind,
    pub order: usize,
    pub locked: bool,
    pub visible: bool,
    pub muted: bool,
    pub solo: bool,
    pub volume: f64, // dB (-60 to +12, default 0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_track_id: Option<String>,
}

impl Track {
    pub fn new_video(id: impl Into<String>, name: impl Into<String>, order: usize) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            kind: TrackKind::Video,
            order,
            locked: false,
            visible: true,
            muted: false,
            solo: false,
            volume: 0.0,
            parent_track_id: None,
        }
    }

    pub fn new_audio(id: impl Into<String>, name: impl Into<String>, order: usize) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            kind: TrackKind::Audio,
            order,
            locked: false,
            visible: true,
            muted: false,
            solo: false,
            volume: 0.0,
            parent_track_id: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemType {
    Video,
    Audio,
    Text,
    Image,
    Shape,
    Composition,
    Adjustment,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub rotation: f64,
    pub opacity: f64,
    pub corner_radius: f64,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 1920.0,
            height: 1080.0,
            rotation: 0.0,
            opacity: 1.0,
            corner_radius: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct CropSettings {
    pub top: f64,
    pub bottom: f64,
    pub left: f64,
    pub right: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimelineItem {
    pub id: String,
    pub track_id: String,
    pub from: i64,               // start frame
    pub duration_in_frames: i64, // duration in frames
    pub label: String,
    pub item_type: ItemType,
    pub src: Option<String>,
    pub media_id: Option<String>,

    // Media boundaries & source trimming
    pub source_start: i64,
    pub source_end: i64,
    pub source_duration: i64,
    pub speed: f64, // Playback speed multiplier (default 1.0)

    // Spatial transforms & crop
    pub transform: Transform,
    pub crop: Option<CropSettings>,
    pub blend_mode: BlendMode,

    // Audio properties
    pub volume: f64,           // Volume in dB (-60 to +12)
    pub audio_fade_in_sec: f64,
    pub audio_fade_out_sec: f64,

    // Video properties
    pub fade_in_sec: f64,
    pub fade_out_sec: f64,

    // Effects stack
    #[serde(default)]
    pub effects: Vec<ItemEffect>,
}

impl TimelineItem {
    pub fn new_video(
        id: impl Into<String>,
        track_id: impl Into<String>,
        from: i64,
        duration: i64,
        src: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            track_id: track_id.into(),
            from,
            duration_in_frames: duration,
            label: "Video Clip".to_string(),
            item_type: ItemType::Video,
            src: Some(src.into()),
            media_id: None,
            source_start: 0,
            source_end: duration,
            source_duration: duration,
            speed: 1.0,
            transform: Transform::default(),
            crop: None,
            blend_mode: BlendMode::Normal,
            volume: 0.0,
            audio_fade_in_sec: 0.0,
            audio_fade_out_sec: 0.0,
            fade_in_sec: 0.0,
            fade_out_sec: 0.0,
            effects: Vec::new(),
        }
    }

    pub fn new_audio(
        id: impl Into<String>,
        track_id: impl Into<String>,
        from: i64,
        duration: i64,
        src: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            track_id: track_id.into(),
            from,
            duration_in_frames: duration,
            label: "Audio Track".to_string(),
            item_type: ItemType::Audio,
            src: Some(src.into()),
            media_id: None,
            source_start: 0,
            source_end: duration,
            source_duration: duration,
            speed: 1.0,
            transform: Transform::default(),
            crop: None,
            blend_mode: BlendMode::Normal,
            volume: 0.0,
            audio_fade_in_sec: 0.0,
            audio_fade_out_sec: 0.0,
            fade_in_sec: 0.0,
            fade_out_sec: 0.0,
            effects: Vec::new(),
        }
    }

    pub fn to_frame(&self) -> i64 {
        self.from + self.duration_in_frames
    }
}
