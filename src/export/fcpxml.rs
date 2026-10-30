use crate::types::project::Project;

pub struct FcpxmlExport;

impl FcpxmlExport {
    pub fn export(project: &Project) -> String {
        let fps = project.metadata.fps;
        let frame_duration = format!("1/{}s", fps.round() as i64);
        let total_dur = format!("{:.3}s", project.total_duration_seconds());

        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<!DOCTYPE fcpxml>\n");
        xml.push_str("<fcpxml version=\"1.9\">\n");
        xml.push_str("  <resources>\n");
        xml.push_str(&format!(
            "    <format id=\"r1\" name=\"FFVideoFormatCustom\" frameDuration=\"{}\" width=\"{}\" height=\"{}\" />\n",
            frame_duration, project.metadata.width, project.metadata.height
        ));

        // Register asset resources for all unique media items
        for (i, item) in project.timeline.items.iter().enumerate() {
            if let Some(ref src) = item.src {
                let asset_id = format!("r{}", i + 2);
                xml.push_str(&format!(
                    "    <asset id=\"{}\" name=\"{}\" src=\"file://{}\" start=\"0s\" duration=\"{}\" hasVideo=\"1\" hasAudio=\"1\" format=\"r1\" />\n",
                    asset_id, item.label, src, total_dur
                ));
            }
        }

        xml.push_str("  </resources>\n");
        xml.push_str("  <library>\n");
        xml.push_str(&format!("    <event name=\"KubeezCut: {}\">\n", project.name));
        xml.push_str(&format!("      <project name=\"{}\">\n", project.name));
        xml.push_str(&format!("        <sequence format=\"r1\" duration=\"{}\">\n", total_dur));
        xml.push_str("          <spine>\n");

        for (i, item) in project.timeline.items.iter().enumerate() {
            let offset_sec = item.from as f64 / fps;
            let dur_sec = item.duration_in_frames as f64 / fps;
            let src_in_sec = item.source_start as f64 / fps;
            let asset_ref = format!("r{}", i + 2);

            xml.push_str(&format!(
                "            <asset-clip ref=\"{}\" name=\"{}\" offset=\"{:.4}s\" start=\"{:.4}s\" duration=\"{:.4}s\" format=\"r1\" />\n",
                asset_ref, item.label, offset_sec, src_in_sec, dur_sec
            ));
        }

        xml.push_str("          </spine>\n");
        xml.push_str("        </sequence>\n");
        xml.push_str("      </project>\n");
        xml.push_str("    </event>\n");
        xml.push_str("  </library>\n");
        xml.push_str("</fcpxml>\n");

        xml
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::timeline::TimelineItem;

    #[test]
    fn test_fcpxml_export() {
        let mut proj = Project::new("p1", "Test Export");
        proj.timeline.items.push(TimelineItem::new_video("v1", "t1", 0, 60, "/path/to/media.mp4"));

        let xml = FcpxmlExport::export(&proj);
        assert!(xml.contains("<fcpxml version=\"1.9\">"));
        assert!(xml.contains("Test Export"));
        assert!(xml.contains("asset-clip ref=\"r2\""));
    }
}
