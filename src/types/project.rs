use crate::types::keyframe::ItemKeyframes;
use crate::types::timeline::{TimelineItem, Track};
use crate::types::transition::Transition;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ProjectResolution {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub background_color: [u8; 4], // RGBA
}

impl Default for ProjectResolution {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            fps: 30.0,
            background_color: [0, 0, 0, 255],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Marker {
    pub id: String,
    pub frame: i64,
    pub label: String,
    pub color: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ProjectTimeline {
    pub tracks: Vec<Track>,
    pub items: Vec<TimelineItem>,
    pub transitions: Vec<Transition>,
    pub keyframes: Vec<ItemKeyframes>,
    pub markers: Vec<Marker>,
    pub in_point: Option<i64>,
    pub out_point: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    pub schema_version: u32,
    pub created_at: i64,
    pub updated_at: i64,
    pub metadata: ProjectResolution,
    pub timeline: ProjectTimeline,
}

impl Project {
    pub const CURRENT_SCHEMA_VERSION: u32 = 6;

    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        let now = Utc::now().timestamp_millis();
        Self {
            id: id.into(),
            name: name.into(),
            description: "Created with KubeezCut".to_string(),
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            created_at: now,
            updated_at: now,
            metadata: ProjectResolution::default(),
            timeline: ProjectTimeline::default(),
        }
    }

    pub fn total_duration_frames(&self) -> i64 {
        self.timeline
            .items
            .iter()
            .map(|item| item.to_frame())
            .max()
            .unwrap_or(0)
    }

    pub fn total_duration_seconds(&self) -> f64 {
        if self.metadata.fps > 0.0 {
            self.total_duration_frames() as f64 / self.metadata.fps
        } else {
            0.0
        }
    }
}
