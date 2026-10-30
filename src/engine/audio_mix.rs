use crate::types::timeline::TimelineItem;

pub struct AudioMixer;

impl AudioMixer {
    /// Convert decibels to linear amplitude: linear = 10^(dB / 20)
    pub fn db_to_linear(db: f64) -> f64 {
        if db <= -60.0 {
            0.0
        } else {
            10.0f64.powf(db / 20.0)
        }
    }

    /// Calculate effective linear audio gain at a specific frame `current_frame`
    /// taking into account base volume, audio fade-in, and audio fade-out.
    pub fn calculate_gain(item: &TimelineItem, current_frame: i64, fps: f64) -> f64 {
        let base_linear = Self::db_to_linear(item.volume);
        if current_frame < item.from || current_frame >= item.to_frame() {
            return 0.0;
        }

        let rel_frame = current_frame - item.from;
        let total_frames = item.duration_in_frames;

        let fade_in_frames = (item.audio_fade_in_sec * fps).round() as i64;
        let fade_out_frames = (item.audio_fade_out_sec * fps).round() as i64;

        let mut multiplier = 1.0;

        // Fade in
        if fade_in_frames > 0 && rel_frame < fade_in_frames {
            multiplier *= rel_frame as f64 / fade_in_frames as f64;
        }

        // Fade out
        let frames_from_end = total_frames - rel_frame;
        if fade_out_frames > 0 && frames_from_end < fade_out_frames {
            multiplier *= frames_from_end as f64 / fade_out_frames as f64;
        }

        (base_linear * multiplier).clamp(0.0, 2.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_to_linear() {
        assert_eq!(AudioMixer::db_to_linear(0.0), 1.0);
        assert!((AudioMixer::db_to_linear(-6.0) - 0.501).abs() < 1e-3);
        assert_eq!(AudioMixer::db_to_linear(-70.0), 0.0);
    }

    #[test]
    fn test_calculate_gain_with_fades() {
        let mut item = TimelineItem::new_audio("a1", "track1", 0, 100, "audio.mp3");
        item.audio_fade_in_sec = 1.0;  // 30 frames
        item.audio_fade_out_sec = 1.0; // 30 frames
        let fps = 30.0;

        assert_eq!(AudioMixer::calculate_gain(&item, 0, fps), 0.0); // start of fade in
        assert!((AudioMixer::calculate_gain(&item, 15, fps) - 0.5).abs() < 1e-4); // mid fade in
        assert_eq!(AudioMixer::calculate_gain(&item, 50, fps), 1.0); // full volume
        assert!((AudioMixer::calculate_gain(&item, 85, fps) - 0.5).abs() < 1e-4); // mid fade out
    }
}
