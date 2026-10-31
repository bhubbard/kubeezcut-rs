use kubeezcut::{
    engine::{AudioMixer, EffectsPipeline, KeyframeEvaluator, TimelineOps},
    export::{
        FcpxmlExport, JsonProject, OtioExport, PremiereExport, RationalTime, RenderConfig,
        RenderOrchestrator, TimeRange, TimeTransform,
    },
    types::{
        BezierControlPoints, EasingConfig, EasingType, EffectPreset, ItemEffect, Keyframe, Project,
        ProjectResolution, SpringParameters, TimelineItem, Track, Transition, TransitionKind,
    },
};

#[test]
fn test_upstream_project_creation_and_duration() {
    let mut project = Project::new("proj-001", "My First Edit");
    project.metadata = ProjectResolution {
        width: 1920,
        height: 1080,
        fps: 30.0,
        background_color: [0, 0, 0, 255],
    };

    project.timeline.tracks.push(Track::new_video("track-v1", "Main Video", 0));
    project.timeline.tracks.push(Track::new_audio("track-a1", "Background Music", 1));

    // Add 2 video items: 0..90 and 90..180
    project.timeline.items.push(TimelineItem::new_video(
        "item-1",
        "track-v1",
        0,
        90,
        "/assets/intro.mp4",
    ));
    project.timeline.items.push(TimelineItem::new_video(
        "item-2",
        "track-v1",
        90,
        90,
        "/assets/climax.mp4",
    ));

    assert_eq!(project.total_duration_frames(), 180);
    assert_eq!(project.total_duration_seconds(), 6.0);
}

#[test]
fn test_upstream_split_preserves_trim_and_speed() {
    let mut item = TimelineItem::new_video("clip_a", "v_track", 30, 120, "video.mp4");
    item.source_start = 10;
    item.source_end = 130;
    item.speed = 1.5;

    // Split at frame 60 (30 frames into the clip)
    let (part1, part2) = TimelineOps::split_item(&item, 60).unwrap();

    assert_eq!(part1.from, 30);
    assert_eq!(part1.duration_in_frames, 30);
    assert_eq!(part1.source_start, 10);
    // 30 frames * 1.5 speed = 45 source frames
    assert_eq!(part1.source_end, 10 + 45);

    assert_eq!(part2.from, 60);
    assert_eq!(part2.duration_in_frames, 90);
    assert_eq!(part2.source_start, 10 + 45);
    assert_eq!(part2.source_end, 130);
}

#[test]
fn test_upstream_trim_in_and_trim_out() {
    let mut item = TimelineItem::new_video("clip", "track", 0, 100, "video.mp4");
    item.source_start = 0;
    item.source_end = 100;

    // Trim In: move start from 0 to 25
    TimelineOps::trim_in(&mut item, 25).unwrap();
    assert_eq!(item.from, 25);
    assert_eq!(item.duration_in_frames, 75);
    assert_eq!(item.source_start, 25);

    // Trim Out: move end from 100 to 80
    TimelineOps::trim_out(&mut item, 80).unwrap();
    assert_eq!(item.from, 25);
    assert_eq!(item.duration_in_frames, 55);
    assert_eq!(item.source_end, 80);
}

#[test]
fn test_upstream_slip_and_slide() {
    let mut item = TimelineItem::new_video("clip", "track", 30, 60, "video.mp4");
    item.source_start = 100;
    item.source_end = 160;

    // Slip: shifts media content by +20 frames without altering timeline positioning
    TimelineOps::slip(&mut item, 20);
    assert_eq!(item.from, 30);
    assert_eq!(item.duration_in_frames, 60);
    assert_eq!(item.source_start, 120);
    assert_eq!(item.source_end, 180);
}

#[test]
fn test_upstream_ripple_delete_gap_consolidation() {
    let mut items = vec![
        TimelineItem::new_video("c1", "track-1", 0, 50, "a.mp4"),
        TimelineItem::new_video("c2", "track-1", 50, 40, "b.mp4"),
        TimelineItem::new_video("c3", "track-1", 90, 60, "c.mp4"),
        TimelineItem::new_audio("a1", "track-2", 0, 200, "music.mp3"), // Audio on different track
    ];

    // Ripple delete c2 (duration 40)
    TimelineOps::ripple_delete(&mut items, "c2").unwrap();

    assert_eq!(items.len(), 3);
    assert_eq!(items[0].id, "c1");
    assert_eq!(items[1].id, "c3");
    assert_eq!(items[1].from, 50); // shifted from 90 to 50
    assert_eq!(items[2].id, "a1");
    assert_eq!(items[2].from, 0);  // unchanged because different track
}

#[test]
fn test_upstream_snap_to_grid() {
    let snap_markers = [0, 60, 120, 180];
    assert_eq!(TimelineOps::snap_frame(58, &snap_markers, 4), 60);
    assert_eq!(TimelineOps::snap_frame(63, &snap_markers, 4), 60);
    assert_eq!(TimelineOps::snap_frame(75, &snap_markers, 4), 75); // outside threshold
}

#[test]
fn test_upstream_keyframe_bezier_and_spring_evaluation() {
    let mut k1 = Keyframe::new("k1", 0, 0.0, EasingType::CubicBezier);
    k1.easing_config = Some(EasingConfig {
        easing: EasingType::CubicBezier,
        bezier: Some(BezierControlPoints { x1: 0.42, y1: 0.0, x2: 0.58, y2: 1.0 }),
        spring: None,
    });

    let k2 = Keyframe::new("k2", 60, 100.0, EasingType::Linear);
    let keyframes = vec![k1, k2];

    let v_start = KeyframeEvaluator::evaluate(&keyframes, 0);
    assert_eq!(v_start, 0.0);

    let v_end = KeyframeEvaluator::evaluate(&keyframes, 60);
    assert_eq!(v_end, 100.0);

    let v_mid = KeyframeEvaluator::evaluate(&keyframes, 30);
    assert!((v_mid - 50.0).abs() < 5.0);

    // Spring evaluation
    let spring = SpringParameters { tension: 170.0, friction: 26.0, mass: 1.0 };
    let s1 = KeyframeEvaluator::solve_spring(1.0, spring);
    assert!((s1 - 1.0).abs() < 0.1);
}

#[test]
fn test_upstream_audio_fades_and_decibels() {
    let mut item = TimelineItem::new_audio("audio1", "track-a", 0, 120, "theme.mp3");
    item.volume = -6.0; // ~0.501
    item.audio_fade_in_sec = 1.0;  // 30 frames
    item.audio_fade_out_sec = 1.0; // 30 frames
    let fps = 30.0;

    let g_start = AudioMixer::calculate_gain(&item, 0, fps);
    assert_eq!(g_start, 0.0);

    let g_mid = AudioMixer::calculate_gain(&item, 60, fps);
    assert!((g_mid - 0.501).abs() < 1e-3);

    let g_end = AudioMixer::calculate_gain(&item, 119, fps);
    assert!(g_end < 0.1);
}

#[test]
fn test_upstream_effects_presets_and_compilation() {
    let preset = EffectPreset::vintage();
    let items = preset
        .effects
        .into_iter()
        .enumerate()
        .map(|(i, eff)| ItemEffect::new(format!("eff_{}", i), eff))
        .collect::<Vec<_>>();

    let filters = EffectsPipeline::compile_effects_to_ffmpeg(&items);
    assert_eq!(filters.len(), 3);
    assert!(filters.iter().any(|f| f.contains("colorchannelmixer=")));
    assert!(filters.iter().any(|f| f.contains("eq=contrast=1.100")));
    assert!(filters.iter().any(|f| f.contains("eq=brightness=-0.100")));
}

#[test]
fn test_upstream_project_serialization_roundtrip() {
    let mut project = Project::new("proj-999", "Complete Feature Project");
    project.metadata = ProjectResolution {
        width: 3840,
        height: 2160,
        fps: 60.0,
        background_color: [10, 10, 10, 255],
    };
    project.timeline.tracks.push(Track::new_video("v1", "Track 1", 0));
    project.timeline.items.push(TimelineItem::new_video("item1", "v1", 0, 120, "clip.mp4"));
    project.timeline.transitions.push(Transition::new(
        "tr1",
        TransitionKind::CrossDissolve,
        "item1",
        "item2",
        30,
    ));

    let json_str = JsonProject::to_string(&project).unwrap();
    let loaded = JsonProject::from_str(&json_str).unwrap();

    assert_eq!(loaded.id, "proj-999");
    assert_eq!(loaded.metadata.width, 3840);
    assert_eq!(loaded.metadata.fps, 60.0);
    assert_eq!(loaded.timeline.transitions.len(), 1);
    assert_eq!(loaded.timeline.transitions[0].kind, TransitionKind::CrossDissolve);
}

#[test]
fn test_upstream_fcpxml_and_premiere_export() {
    let mut proj = Project::new("p_export", "Exchanger");
    proj.timeline.items.push(TimelineItem::new_video("v1", "t1", 0, 90, "render.mp4"));

    let fcpxml = FcpxmlExport::export(&proj);
    assert!(fcpxml.contains("<fcpxml version=\"1.9\">"));
    assert!(fcpxml.contains("asset-clip ref=\"r2\""));

    let premiere = PremiereExport::export(&proj);
    assert!(premiere.contains("<xmeml version=\"4\">"));
    assert!(premiere.contains("<clipitem id=\"clip-1\">"));
}

#[test]
fn test_upstream_headless_render_pipeline() {
    let mut proj = Project::new("p_render", "Headless Composition");
    proj.timeline.items.push(TimelineItem::new_video("v1", "t1", 0, 60, "input.mov"));

    let cfg = RenderConfig::default();
    let cmd = RenderOrchestrator::build_ffmpeg_command(&proj, &cfg).unwrap();

    assert_eq!(cmd[0], "ffmpeg");
    assert!(cmd.iter().any(|c| c.contains("concat=n=1")));
    assert!(cmd.iter().any(|c| c == "+faststart"));
}

#[test]
fn test_upstream_otio_edl_and_rational_time() {
    let rt = RationalTime::new(180.0, 60.0);
    assert_eq!(rt.to_seconds(), 3.0);
    assert_eq!(rt.to_smpte_timecode(false), "00:00:03:00");

    let tr = TimeRange::new(RationalTime::new(0.0, 24.0), RationalTime::new(48.0, 24.0));
    assert_eq!(tr.end_time_exclusive().to_frames(), 48);

    let xform = TimeTransform::new(RationalTime::new(24.0, 24.0), 2.0, 24.0);
    let shifted = xform.applied_to_time(RationalTime::new(10.0, 24.0));
    assert_eq!(shifted.to_frames(), 44);

    let mut proj = Project::new("p_otio", "ASWF OpenTimelineIO Sequence");
    proj.timeline
        .tracks
        .push(Track::new_video("v1", "Video 1", 0));
    proj.timeline.items.push(TimelineItem::new_video(
        "c1",
        "v1",
        0,
        90,
        "/media/scene01.mov",
    ));

    let otio = OtioExport::export_otio_json(&proj);
    assert!(otio.contains("\"OTIO_SCHEMA\": \"Timeline.1\""));
    assert!(otio.contains("\"OTIO_SCHEMA\": \"Clip.2\""));

    let edl = OtioExport::export_cmx3600_edl(&proj);
    assert!(edl.contains("TITLE: ASWF OpenTimelineIO Sequence"));
    assert!(edl.contains("* FROM CLIP NAME: Video Clip"));

    let csv = OtioExport::export_resolve_csv(&proj);
    assert!(csv.contains("Event,Reel,Track,Clip Name"));
}
