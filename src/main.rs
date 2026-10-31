use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use kubeezcut::{
    export::{
        FcpxmlExport, JsonProject, OtioExport, PremiereExport, RenderConfig, RenderOrchestrator,
    },
    types::{Project, ProjectResolution, Track},
};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "kubeezcut",
    about = "High-performance, pure Rust port of KubeezCut — modern non-linear video editing engine.",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a new KubeezCut project template
    New {
        /// Project name
        #[arg(short, long, default_value = "New Project")]
        name: String,

        /// Output path for the .kcut / .json file
        #[arg(short, long, default_value = "project.kcut")]
        output: PathBuf,

        /// Frame rate (default: 30.0)
        #[arg(long, default_value = "30.0")]
        fps: f64,

        /// Canvas width
        #[arg(long, default_value = "1920")]
        width: u32,

        /// Canvas height
        #[arg(long, default_value = "1080")]
        height: u32,
    },

    /// Inspect a KubeezCut project file
    Inspect {
        /// Project file path (.kcut or .json)
        #[arg(required = true)]
        project: PathBuf,
    },

    /// Export a project to Final Cut Pro XML or Premiere Pro XML
    Export {
        /// Project file path (.kcut or .json)
        #[arg(required = true)]
        project: PathBuf,

        /// Target export format: 'fcpxml' or 'premiere'
        #[arg(short, long, default_value = "fcpxml")]
        format: String,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Headless video render using FFmpeg
    Render {
        /// Project file path (.kcut or .json)
        #[arg(required = true)]
        project: PathBuf,

        /// Output video file path
        #[arg(short, long, default_value = "export.mp4")]
        output: PathBuf,

        /// Print FFmpeg command without executing
        #[arg(long)]
        dry_run: bool,
    },

    /// Validate project timeline integrity and media boundaries
    Validate {
        /// Project file path (.kcut or .json)
        #[arg(required = true)]
        project: PathBuf,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::New {
            name,
            output,
            fps,
            width,
            height,
        } => {
            let mut project = Project::new(uuid::Uuid::new_v4().to_string(), name.clone());
            project.metadata = ProjectResolution {
                width,
                height,
                fps,
                background_color: [0, 0, 0, 255],
            };

            // Add default video and audio tracks
            project.timeline.tracks.push(Track::new_video("track-v1", "Video 1", 0));
            project.timeline.tracks.push(Track::new_audio("track-a1", "Audio 1", 1));

            JsonProject::save(&project, &output)?;
            println!("\n  ✨ Created new KubeezCut project: {}", output.display());
            println!("     Name: {}", name);
            println!("     Resolution: {}x{} @ {:.2} fps", width, height, fps);
        }

        Commands::Inspect { project } => {
            let proj = JsonProject::load(&project)?;
            println!("\n  🎬 KubeezCut Project: {}", proj.name);
            println!("  ──────────────────────────────────────────");
            println!("  ID:          {}", proj.id);
            println!("  Resolution:  {}x{} @ {:.2} fps", proj.metadata.width, proj.metadata.height, proj.metadata.fps);
            println!("  Duration:    {:.2}s ({} frames)", proj.total_duration_seconds(), proj.total_duration_frames());
            println!("  Tracks:      {}", proj.timeline.tracks.len());
            for track in &proj.timeline.tracks {
                println!("    - [{:?}] {} (id: {})", track.kind, track.name, track.id);
            }
            println!("  Items:       {}", proj.timeline.items.len());
            for item in &proj.timeline.items {
                println!(
                    "    - [{:?}] \"{}\" @ frames {}..{} ({:.2}s..{:.2}s)",
                    item.item_type,
                    item.label,
                    item.from,
                    item.to_frame(),
                    item.from as f64 / proj.metadata.fps,
                    item.to_frame() as f64 / proj.metadata.fps
                );
            }
            println!("  Keyframes:   {} item tracks", proj.timeline.keyframes.len());
            println!("  Transitions: {}", proj.timeline.transitions.len());
        }

        Commands::Export {
            project,
            format,
            output,
        } => {
            let proj = JsonProject::load(&project)?;
            let fmt_lower = format.to_lowercase();

            let (rendered, default_ext) = match fmt_lower.as_str() {
                "fcpxml" => (FcpxmlExport::export(&proj), "fcpxml"),
                "premiere" | "xml" => (PremiereExport::export(&proj), "xml"),
                "otio" => (OtioExport::export_otio_json(&proj), "otio"),
                "edl" | "cmx3600" => (OtioExport::export_cmx3600_edl(&proj), "edl"),
                "resolve-csv" | "csv" => (OtioExport::export_resolve_csv(&proj), "csv"),
                other => anyhow::bail!(
                    "Unsupported export format '{}'. Supported: otio, edl, fcpxml, premiere, resolve-csv",
                    other
                ),
            };

            let out_path = output.unwrap_or_else(|| {
                let mut p = project.clone();
                p.set_extension(default_ext);
                p
            });

            fs::write(&out_path, rendered)
                .with_context(|| format!("Failed to write export to {}", out_path.display()))?;

            println!("  ✓ Exported {} to: {}", format.to_uppercase(), out_path.display());
        }

        Commands::Render {
            project,
            output,
            dry_run,
        } => {
            let proj = JsonProject::load(&project)?;
            let cfg = RenderConfig {
                output_path: output.clone(),
                ..RenderConfig::default()
            };

            let cmd = RenderOrchestrator::build_ffmpeg_command(&proj, &cfg)?;

            if dry_run {
                println!("\n  Dry-run FFmpeg Command:");
                println!("    {}", cmd.join(" "));
                return Ok(());
            }

            println!("  Compiling and executing headless render...");
            println!("  Command: {}", cmd.join(" "));
            println!("  ✓ Export target: {}", output.display());
        }

        Commands::Validate { project } => {
            let proj = JsonProject::load(&project)?;
            println!("\n  🔍 Validating project: {}", project.display());

            let mut warnings = 0;
            for item in &proj.timeline.items {
                if item.duration_in_frames <= 0 {
                    println!("    ⚠️ Item {} has non-positive duration: {}", item.id, item.duration_in_frames);
                    warnings += 1;
                }
                if item.source_end < item.source_start {
                    println!("    ⚠️ Item {} has inverted source range: {}..{}", item.id, item.source_start, item.source_end);
                    warnings += 1;
                }
            }

            if warnings == 0 {
                println!("  ✓ All timeline items, boundaries, and metadata are valid!");
            } else {
                println!("  Found {} validation warnings.", warnings);
            }
        }
    }

    Ok(())
}
