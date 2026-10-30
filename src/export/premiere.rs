use crate::types::project::Project;

pub struct PremiereExport;

impl PremiereExport {
    pub fn export(project: &Project) -> String {
        let fps_int = project.metadata.fps.round() as i64;
        let total_frames = project.total_duration_frames();

        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<!DOCTYPE xmeml>\n");
        xml.push_str("<xmeml version=\"4\">\n");
        xml.push_str("  <sequence>\n");
        xml.push_str(&format!("    <name>{}</name>\n", project.name));
        xml.push_str(&format!("    <duration>{}</duration>\n", total_frames));
        xml.push_str("    <rate>\n");
        xml.push_str(&format!("      <timebase>{}</timebase>\n", fps_int));
        xml.push_str("      <ntsc>FALSE</ntsc>\n");
        xml.push_str("    </rate>\n");
        xml.push_str("    <media>\n");
        xml.push_str("      <video>\n");
        xml.push_str("        <format>\n");
        xml.push_str("          <samplecharacteristics>\n");
        xml.push_str(&format!("            <rate><timebase>{}</timebase></rate>\n", fps_int));
        xml.push_str(&format!("            <width>{}</width>\n", project.metadata.width));
        xml.push_str(&format!("            <height>{}</height>\n", project.metadata.height));
        xml.push_str("          </samplecharacteristics>\n");
        xml.push_str("        </format>\n");
        xml.push_str("        <track>\n");

        for (i, item) in project.timeline.items.iter().enumerate() {
            let in_frame = item.source_start;
            let out_frame = item.source_end;
            let dur = item.duration_in_frames;
            let src_path = item.src.as_deref().unwrap_or("media.mp4");

            xml.push_str(&format!("          <clipitem id=\"clip-{}\">\n", i + 1));
            xml.push_str(&format!("            <name>{}</name>\n", item.label));
            xml.push_str(&format!("            <duration>{}</duration>\n", dur));
            xml.push_str(&format!("            <start>{}</start>\n", item.from));
            xml.push_str(&format!("            <end>{}</end>\n", item.to_frame()));
            xml.push_str(&format!("            <in>{}</in>\n", in_frame));
            xml.push_str(&format!("            <out>{}</out>\n", out_frame));
            xml.push_str("            <file id=\"file-1\">\n");
            xml.push_str(&format!("              <name>{}</name>\n", item.label));
            xml.push_str(&format!("              <pathurl>file://{}</pathurl>\n", src_path));
            xml.push_str("              <rate>\n");
            xml.push_str(&format!("                <timebase>{}</timebase>\n", fps_int));
            xml.push_str("              </rate>\n");
            xml.push_str("            </file>\n");
            xml.push_str("          </clipitem>\n");
        }

        xml.push_str("        </track>\n");
        xml.push_str("      </video>\n");
        xml.push_str("    </media>\n");
        xml.push_str("  </sequence>\n");
        xml.push_str("</xmeml>\n");

        xml
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::timeline::TimelineItem;

    #[test]
    fn test_premiere_export() {
        let mut proj = Project::new("p1", "Premiere Seq");
        proj.timeline.items.push(TimelineItem::new_video("v1", "t1", 0, 90, "media.mp4"));

        let xml = PremiereExport::export(&proj);
        assert!(xml.contains("<xmeml version=\"4\">"));
        assert!(xml.contains("<name>Premiere Seq</name>"));
        assert!(xml.contains("<clipitem id=\"clip-1\">"));
    }
}
