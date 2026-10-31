//! # KubeezCut (Rust Edition) 🦀🎬
//!
//! A high-performance, native pure Rust port of [KubeezMedia/KubeezCut](https://github.com/KubeezMedia/KubeezCut) —
//! modern non-linear video editing (NLE) engine, multi-track timeline, WebGPU/shader effects pipeline,
//! keyframe Bezier animation, and headless render export.

pub mod engine;
pub mod export;
pub mod types;

pub use engine::{AudioMixer, EffectsPipeline, KeyframeEvaluator, TimelineOps};
pub use export::{
    detect_encoder, AafCompositionPackage, AdForgeTimeline, FcpxmlExport, JsonProject,
    MatchCutCandidate, MatchCutEngine, MetricProjectionHead, OtioExport, PremiereExport,
    RationalTime, RenderConfig, RenderOrchestrator, Sb37ComplianceEngine, Sb37DisclaimerCue,
    ShotEmbedding, TakeCandidate, TimeRange, TimeTransform,
};
pub use types::{
    AnimatableProperty, BezierControlPoints, BlendMode, CropSettings, EasingConfig, EasingType,
    EffectPreset, GpuEffect, ItemEffect, ItemKeyframes, ItemType, Keyframe, Marker, Project,
    ProjectResolution, ProjectTimeline, PropertyKeyframes, SpringParameters, TimelineItem, Track,
    TrackKind, Transform, Transition, TransitionAlignment, TransitionKind,
};
