use crate::engine::effects_pipeline::EffectsPipeline;
use crate::types::project::Project;
use anyhow::{bail, Result};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct RenderConfig {
    pub output_path: PathBuf,
    pub video_codec: String,
    pub audio_codec: String,
    pub crf: u32,
    pub preset: String,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            output_path: PathBuf::from("export.mp4"),
            video_codec: detect_encoder(),
            audio_codec: "aac".to_string(),
            crf: 18,
            preset: "fast".to_string(),
        }
    }
}

pub fn detect_encoder() -> String {
    if cfg!(target_os = "macos") {
        "h264_videotoolbox".to_string()
    } else {
        "libx264".to_string()
    }
}

pub struct RenderOrchestrator;

impl RenderOrchestrator {
    /// Compiles a KubeezCut project into an FFmpeg execution command
    pub fn build_ffmpeg_command(project: &Project, config: &RenderConfig) -> Result<Vec<String>> {
        let items = &project.timeline.items;
        if items.is_empty() {
            bail!("Timeline contains no items to render");
        }

        let fps = project.metadata.fps;
        let mut cmd = vec!["ffmpeg".to_string(), "-y".to_string()];

        // Inputs
        for item in items {
            if let Some(ref src) = item.src {
                cmd.push("-i".to_string());
                cmd.push(src.clone());
            }
        }

        let mut filter_complex = String::new();
        let mut v_outputs = Vec::new();

        for (i, item) in items.iter().enumerate() {
            let start_sec = item.source_start as f64 / fps;
            let dur_sec = item.duration_in_frames as f64 / fps;

            let mut chain = format!(
                "[{}:v]trim=start={:.3}:duration={:.3},setpts=PTS-STARTPTS",
                i, start_sec, dur_sec
            );

            // Add compiled effects
            let effect_filters = EffectsPipeline::compile_effects_to_ffmpeg(&item.effects);
            if !effect_filters.is_empty() {
                chain.push(',');
                chain.push_str(&effect_filters.join(","));
            }

            // Video fade in / out
            if item.fade_in_sec > 0.0 {
                chain.push_str(&format!(",fade=t=in:st=0:d={:.3}", item.fade_in_sec));
            }
            if item.fade_out_sec > 0.0 {
                let out_start = dur_sec - item.fade_out_sec;
                chain.push_str(&format!(",fade=t=out:st={:.3}:d={:.3}", out_start, item.fade_out_sec));
            }

            let out_label = format!("v{}", i);
            chain.push_str(&format!("[{}];", out_label));
            filter_complex.push_str(&chain);
            v_outputs.push(format!("[{}]", out_label));
        }

        // Concat sequence
        filter_complex.push_str(&format!(
            "{}concat=n={}:v=1:a=0[outv]",
            v_outputs.join(""),
            items.len()
        ));

        cmd.push("-filter_complex".to_string());
        cmd.push(filter_complex);

        cmd.push("-map".to_string());
        cmd.push("[outv]".to_string());

        cmd.push("-c:v".to_string());
        cmd.push(config.video_codec.clone());

        if config.video_codec == "libx264" {
            cmd.push("-preset".to_string());
            cmd.push(config.preset.clone());
            cmd.push("-crf".to_string());
            cmd.push(config.crf.to_string());
        }

        cmd.push("-movflags".to_string());
        cmd.push("+faststart".to_string());

        cmd.push(config.output_path.to_string_lossy().to_string());

        Ok(cmd)
    }

    /// Execute the compiled FFmpeg command
    pub fn execute(cmd: &[String]) -> Result<()> {
        if cmd.is_empty() {
            bail!("Empty FFmpeg command");
        }
        let status = Command::new(&cmd[0]).args(&cmd[1..]).status()?;
        if !status.success() {
            bail!("FFmpeg exited with error status: {}", status);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::timeline::TimelineItem;

    #[test]
    fn test_build_render_command() {
        let mut proj = Project::new("p1", "Render Demo");
        proj.timeline.items.push(TimelineItem::new_video("v1", "t1", 0, 90, "clip.mp4"));

        let cfg = RenderConfig::default();
        let cmd = RenderOrchestrator::build_ffmpeg_command(&proj, &cfg).unwrap();

        assert_eq!(cmd[0], "ffmpeg");
        assert!(cmd.iter().any(|c| c == "-filter_complex"));
        assert!(cmd.iter().any(|c| c == "+faststart"));
    }
}
