use crate::types::timeline::TimelineItem;
use anyhow::{bail, Result};

pub struct TimelineOps;

impl TimelineOps {
    /// Splits a timeline item at a specific frame into two items.
    /// Preserves media boundaries, trims, speed, and spatial transforms.
    pub fn split_item(item: &TimelineItem, split_frame: i64) -> Result<(TimelineItem, TimelineItem)> {
        let item_start = item.from;
        let item_end = item.to_frame();

        if split_frame <= item_start || split_frame >= item_end {
            bail!(
                "Split frame {} is outside of item boundaries ({}..{})",
                split_frame,
                item_start,
                item_end
            );
        }

        let first_duration = split_frame - item_start;
        let second_duration = item_end - split_frame;

        // Calculate media source offsets based on speed
        let source_delta = (first_duration as f64 * item.speed).round() as i64;

        let mut first = item.clone();
        first.id = format!("{}_part1", item.id);
        first.duration_in_frames = first_duration;
        first.source_end = item.source_start + source_delta;

        let mut second = item.clone();
        second.id = format!("{}_part2", item.id);
        second.from = split_frame;
        second.duration_in_frames = second_duration;
        second.source_start = item.source_start + source_delta;

        Ok((first, second))
    }

    /// Trims the start of an item (Trim In)
    pub fn trim_in(item: &mut TimelineItem, new_start_frame: i64) -> Result<()> {
        let item_end = item.to_frame();
        if new_start_frame >= item_end {
            bail!("New start frame cannot be >= item end frame");
        }

        let delta = new_start_frame - item.from;
        let source_delta = (delta as f64 * item.speed).round() as i64;

        item.from = new_start_frame;
        item.duration_in_frames = item_end - new_start_frame;
        item.source_start += source_delta;

        Ok(())
    }

    /// Trims the end of an item (Trim Out)
    pub fn trim_out(item: &mut TimelineItem, new_end_frame: i64) -> Result<()> {
        if new_end_frame <= item.from {
            bail!("New end frame cannot be <= item start frame");
        }

        let new_duration = new_end_frame - item.from;
        let source_delta = (new_duration as f64 * item.speed).round() as i64;

        item.duration_in_frames = new_duration;
        item.source_end = item.source_start + source_delta;

        Ok(())
    }

    /// Performs a Slip Edit: shifts media in/out source frames without changing timeline position or duration
    pub fn slip(item: &mut TimelineItem, source_offset_frames: i64) {
        item.source_start += source_offset_frames;
        item.source_end += source_offset_frames;
    }

    /// Ripple delete: removes the item with `target_id` from the track and shifts
    /// all subsequent items on the same track to close the gap.
    pub fn ripple_delete(items: &mut Vec<TimelineItem>, target_id: &str) -> Result<()> {
        let target_idx = items.iter().position(|i| i.id == target_id);
        let idx = match target_idx {
            Some(i) => i,
            None => bail!("Item with id '{}' not found", target_id),
        };

        let removed = items.remove(idx);
        let gap_duration = removed.duration_in_frames;
        let removed_end = removed.to_frame();
        let track_id = removed.track_id;

        for item in items.iter_mut() {
            if item.track_id == track_id && item.from >= removed_end {
                item.from -= gap_duration;
            }
        }

        Ok(())
    }

    /// Snaps a candidate frame position to the closest edge (within `threshold` frames)
    pub fn snap_frame(candidate: i64, snap_points: &[i64], threshold: i64) -> i64 {
        let mut closest = candidate;
        let mut min_dist = threshold + 1;

        for &p in snap_points {
            let dist = (candidate - p).abs();
            if dist <= threshold && dist < min_dist {
                min_dist = dist;
                closest = p;
            }
        }

        closest
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_item() {
        let item = TimelineItem::new_video("clip1", "track1", 0, 100, "video.mp4");
        let (first, second) = TimelineOps::split_item(&item, 40).unwrap();

        assert_eq!(first.from, 0);
        assert_eq!(first.duration_in_frames, 40);
        assert_eq!(first.source_start, 0);
        assert_eq!(first.source_end, 40);

        assert_eq!(second.from, 40);
        assert_eq!(second.duration_in_frames, 60);
        assert_eq!(second.source_start, 40);
        assert_eq!(second.source_end, 100);
    }

    #[test]
    fn test_trim_in_and_out() {
        let mut item = TimelineItem::new_video("clip1", "track1", 10, 90, "video.mp4");
        TimelineOps::trim_in(&mut item, 20).unwrap();
        assert_eq!(item.from, 20);
        assert_eq!(item.duration_in_frames, 80);
        assert_eq!(item.source_start, 10);

        TimelineOps::trim_out(&mut item, 80).unwrap();
        assert_eq!(item.duration_in_frames, 60);
        assert_eq!(item.to_frame(), 80);
    }

    #[test]
    fn test_ripple_delete() {
        let mut items = vec![
            TimelineItem::new_video("c1", "t1", 0, 30, "a.mp4"),
            TimelineItem::new_video("c2", "t1", 30, 30, "b.mp4"),
            TimelineItem::new_video("c3", "t1", 60, 40, "c.mp4"),
        ];

        TimelineOps::ripple_delete(&mut items, "c2").unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].id, "c1");
        assert_eq!(items[1].id, "c3");
        assert_eq!(items[1].from, 30); // shifted back by 30 frames
    }

    #[test]
    fn test_snap_frame() {
        let snap_points = [0, 30, 60, 90];
        assert_eq!(TimelineOps::snap_frame(29, &snap_points, 3), 30);
        assert_eq!(TimelineOps::snap_frame(45, &snap_points, 3), 45); // outside threshold
    }
}
