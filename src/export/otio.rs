use crate::types::project::Project;
use crate::types::timeline::{ItemType, TrackKind};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Academy Software Foundation OpenTimelineIO `RationalTime.1` representation
/// supporting exact fractional frame rates (23.976, 24, 25, 29.97 DF/NDF, 59.94, 60 fps).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RationalTime {
    pub value: f64,
    pub rate: f64,
}

impl RationalTime {
    pub fn new(value: f64, rate: f64) -> Self {
        Self {
            value,
            rate: if rate <= 0.0 { 24.0 } else { rate },
        }
    }

    pub fn to_seconds(&self) -> f64 {
        self.value / self.rate
    }

    pub fn to_frames(&self) -> i64 {
        self.value.round() as i64
    }

    pub fn rescaled_to(&self, new_rate: f64) -> Self {
        let safe_rate = if new_rate <= 0.0 { 24.0 } else { new_rate };
        Self {
            value: self.to_seconds() * safe_rate,
            rate: safe_rate,
        }
    }

    /// Formats `RationalTime` as SMPTE timecode (`HH:MM:SS:FF` or `HH:MM:SS;FF` for drop-frame).
    pub fn to_smpte_timecode(&self, drop_frame: bool) -> String {
        let nominal_fps = self.rate.round().max(1.0) as i64;
        let total_frames = self.value.round().max(0.0) as i64;
        let sep = if drop_frame { ';' } else { ':' };

        let ff = total_frames % nominal_fps;
        let total_seconds = total_frames / nominal_fps;
        let ss = total_seconds % 60;
        let mm = (total_seconds / 60) % 60;
        let hh = total_seconds / 3600;

        format!("{:02}:{:02}:{:02}{}{:02}", hh, mm, ss, sep, ff)
    }

    pub fn to_otio_value(&self) -> serde_json::Value {
        json!({
            "OTIO_SCHEMA": "RationalTime.1",
            "value": self.value,
            "rate": self.rate
        })
    }
}

/// OpenTimelineIO `TimeRange.1` representation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TimeRange {
    pub start_time: RationalTime,
    pub duration: RationalTime,
}

impl TimeRange {
    pub fn new(start_time: RationalTime, duration: RationalTime) -> Self {
        Self {
            start_time,
            duration,
        }
    }

    pub fn end_time_exclusive(&self) -> RationalTime {
        let dur_rescaled = self.duration.rescaled_to(self.start_time.rate);
        RationalTime::new(
            self.start_time.value + dur_rescaled.value,
            self.start_time.rate,
        )
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        let a_start = self.start_time.to_seconds();
        let a_end = self.end_time_exclusive().to_seconds();
        let b_start = other.start_time.to_seconds();
        let b_end = other.end_time_exclusive().to_seconds();
        a_start < b_end && b_start < a_end
    }

    pub fn to_otio_value(&self) -> serde_json::Value {
        json!({
            "OTIO_SCHEMA": "TimeRange.1",
            "start_time": self.start_time.to_otio_value(),
            "duration": self.duration.to_otio_value()
        })
    }
}

/// OpenTimelineIO `TimeTransform.1` representation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TimeTransform {
    pub offset: RationalTime,
    pub scale: f64,
    pub rate: f64,
}

impl TimeTransform {
    pub fn new(offset: RationalTime, scale: f64, rate: f64) -> Self {
        Self {
            offset,
            scale,
            rate,
        }
    }

    pub fn applied_to_time(&self, t: RationalTime) -> RationalTime {
        let target_rate = if self.rate > 0.0 { self.rate } else { t.rate };
        let rescaled = t.rescaled_to(target_rate);
        let offset_rescaled = self.offset.rescaled_to(target_rate);
        RationalTime::new(
            rescaled.value * self.scale + offset_rescaled.value,
            target_rate,
        )
    }
}

pub struct OtioExport;

impl OtioExport {
    /// Exports a `Project` into Academy Software Foundation OpenTimelineIO (`.otio`) JSON schema (`Timeline.1`)
    pub fn export_otio_json(project: &Project) -> String {
        let fps = project.metadata.fps.max(1.0);
        let mut tracks_json = Vec::new();

        for track in &project.timeline.tracks {
            let mut track_items: Vec<_> = project
                .timeline
                .items
                .iter()
                .filter(|item| item.track_id == track.id)
                .collect();
            track_items.sort_by_key(|item| item.from);

            let mut children = Vec::new();
            let mut cursor: i64 = 0;

            for item in track_items {
                if item.from > cursor {
                    let gap_frames = (item.from - cursor) as f64;
                    let gap_range = TimeRange::new(
                        RationalTime::new(0.0, fps),
                        RationalTime::new(gap_frames, fps),
                    );
                    children.push(json!({
                        "OTIO_SCHEMA": "Gap.1",
                        "name": "Gap",
                        "source_range": gap_range.to_otio_value(),
                        "effects": [],
                        "markers": []
                    }));
                }

                let src_range = TimeRange::new(
                    RationalTime::new(item.source_start as f64, fps),
                    RationalTime::new(item.duration_in_frames as f64, fps),
                );
                let target_url = item.src.as_deref().unwrap_or("offline://generator");

                children.push(json!({
                    "OTIO_SCHEMA": "Clip.2",
                    "name": item.label,
                    "source_range": src_range.to_otio_value(),
                    "media_reference": {
                        "OTIO_SCHEMA": "ExternalReference.1",
                        "target_url": target_url,
                        "available_range": TimeRange::new(
                            RationalTime::new(0.0, fps),
                            RationalTime::new(item.source_duration.max(item.duration_in_frames) as f64, fps)
                        ).to_otio_value()
                    },
                    "effects": [],
                    "markers": []
                }));

                cursor = item.to_frame().max(cursor);
            }

            let kind_str = match track.kind {
                TrackKind::Video => "Video",
                TrackKind::Audio => "Audio",
            };

            tracks_json.push(json!({
                "OTIO_SCHEMA": "Track.1",
                "name": track.name,
                "kind": kind_str,
                "children": children,
                "effects": [],
                "markers": []
            }));
        }

        let markers_json: Vec<_> = project
            .timeline
            .markers
            .iter()
            .map(|m| {
                json!({
                    "OTIO_SCHEMA": "Marker.2",
                    "name": m.label,
                    "color": m.color.to_uppercase(),
                    "marked_range": TimeRange::new(
                        RationalTime::new(m.frame as f64, fps),
                        RationalTime::new(1.0, fps)
                    ).to_otio_value()
                })
            })
            .collect();

        let root = json!({
            "OTIO_SCHEMA": "Timeline.1",
            "name": project.name,
            "global_start_time": RationalTime::new(0.0, fps).to_otio_value(),
            "tracks": {
                "OTIO_SCHEMA": "Stack.1",
                "name": "tracks",
                "children": tracks_json,
                "effects": [],
                "markers": markers_json
            }
        });

        serde_json::to_string_pretty(&root).unwrap_or_else(|_| "{}".to_string())
    }

    /// Exports a `Project` into SMPTE CMX 3600 Edit Decision List (`.edl`) format
    pub fn export_cmx3600_edl(project: &Project) -> String {
        let fps = project.metadata.fps.max(1.0);
        let is_drop = (fps - 29.97).abs() < 0.05 || (fps - 59.94).abs() < 0.05;
        let fcm = if is_drop {
            "DROP FRAME"
        } else {
            "NON-DROP FRAME"
        };

        let mut edl = String::new();
        edl.push_str(&format!("TITLE: {}\n", project.name));
        edl.push_str(&format!("FCM: {}\n\n", fcm));

        let mut sorted_items = project.timeline.items.clone();
        sorted_items.sort_by_key(|i| i.from);

        for (idx, item) in sorted_items.iter().enumerate() {
            let event_num = idx + 1;
            let ch = match item.item_type {
                ItemType::Audio => "A    ",
                _ => "V    ",
            };

            let src_in = RationalTime::new(item.source_start as f64, fps).to_smpte_timecode(is_drop);
            let src_out = RationalTime::new(item.source_end as f64, fps).to_smpte_timecode(is_drop);
            let rec_in = RationalTime::new(item.from as f64, fps).to_smpte_timecode(is_drop);
            let rec_out = RationalTime::new(item.to_frame() as f64, fps).to_smpte_timecode(is_drop);

            edl.push_str(&format!(
                "{:03}  AX       {} C        {} {} {} {}\n",
                event_num, ch, src_in, src_out, rec_in, rec_out
            ));
            edl.push_str(&format!("* FROM CLIP NAME: {}\n", item.label));
            if let Some(ref src) = item.src {
                edl.push_str(&format!("* SOURCE FILE: {}\n", src));
            }
            edl.push('\n');
        }

        edl
    }

    /// Exports a `Project` into DaVinci Resolve Timeline Edit Index CSV format
    pub fn export_resolve_csv(project: &Project) -> String {
        let fps = project.metadata.fps.max(1.0);
        let mut csv = String::from(
            "Event,Reel,Track,Clip Name,Source In,Source Out,Record In,Record Out,Speed\n",
        );

        let mut sorted_items = project.timeline.items.clone();
        sorted_items.sort_by_key(|i| i.from);

        for (idx, item) in sorted_items.iter().enumerate() {
            let src_in = RationalTime::new(item.source_start as f64, fps).to_smpte_timecode(false);
            let src_out = RationalTime::new(item.source_end as f64, fps).to_smpte_timecode(false);
            let rec_in = RationalTime::new(item.from as f64, fps).to_smpte_timecode(false);
            let rec_out = RationalTime::new(item.to_frame() as f64, fps).to_smpte_timecode(false);

            csv.push_str(&format!(
                "{},AX,{},\"{}\",{},{},{},{},{:.2}\n",
                idx + 1,
                item.track_id,
                item.label.replace('"', "'"),
                src_in,
                src_out,
                rec_in,
                rec_out,
                item.speed
            ));
        }

        csv
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::timeline::{TimelineItem, Track};

    #[test]
    fn test_rational_time_and_smpte_timecode() {
        let rt = RationalTime::new(90.0, 30.0);
        assert_eq!(rt.to_seconds(), 3.0);
        assert_eq!(rt.to_smpte_timecode(false), "00:00:03:00");

        let rescaled = rt.rescaled_to(24.0);
        assert_eq!(rescaled.to_frames(), 72);
    }

    #[test]
    fn test_otio_and_cmx3600_edl_exports() {
        let mut proj = Project::new("p1", "OTIO Master");
        proj.timeline
            .tracks
            .push(Track::new_video("v1", "V1", 0));
        proj.timeline.items.push(TimelineItem::new_video(
            "c1",
            "v1",
            0,
            60,
            "/media/shot01.mov",
        ));

        let otio = OtioExport::export_otio_json(&proj);
        assert!(otio.contains("\"OTIO_SCHEMA\": \"Timeline.1\""));
        assert!(otio.contains("\"OTIO_SCHEMA\": \"Clip.2\""));

        let edl = OtioExport::export_cmx3600_edl(&proj);
        assert!(edl.contains("TITLE: OTIO Master"));
        assert!(edl.contains("001  AX       V     C        00:00:00:00 00:00:02:00"));
    }
}
