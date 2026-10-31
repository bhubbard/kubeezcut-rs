# `kubeezcut-rs` 🦀🎬

[![CI](https://github.com/bhubbard/kubeezcut-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/bhubbard/kubeezcut-rs/actions/workflows/ci.yml)
[![Coverage](https://github.com/bhubbard/kubeezcut-rs/actions/workflows/coverage.yml/badge.svg)](https://github.com/bhubbard/kubeezcut-rs/actions/workflows/coverage.yml)
[![Pages](https://github.com/bhubbard/kubeezcut-rs/actions/workflows/pages.yml/badge.svg)](https://code.brandonhubbard.com/kubeezcut-rs/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A high-performance, native pure **Rust** port of [KubeezMedia/KubeezCut](https://github.com/KubeezMedia/KubeezCut) with built-in **Academy Software Foundation OpenTimelineIO (`.otio`)** and **SMPTE CMX 3600 EDL** interchange — delivering a modern non-linear video editing (NLE) timeline engine, WebGPU/shader effects pipeline, Cubic Bezier & damped spring keyframe animation solver, and headless render export.

> **Interactive Studio & Live NLE Workbench**: [**https://code.brandonhubbard.com/kubeezcut-rs/**](https://code.brandonhubbard.com/kubeezcut-rs/)  
> **Empirical Benchmarks**: [**BENCHMARKS.md**](BENCHMARKS.md) (**49.2x faster** than TypeScript / Python OpenTimelineIO, **4.8 MB** peak RSS)

---

## ⚡ Highlights & Architecture

- **🎞️ Multi-Track Non-Linear Timeline Engine (`src/engine/timeline_ops.rs`)**:
  - Video and Audio tracks with decibel volume control (`-60 dB` to `+12 dB`), mute, solo, and lock states.
  - Non-destructive clip editing: **Split at playhead**, **Ripple delete**, **Trim In / Trim Out**, and **Slip / Slide editing** preserving variable playback speed (`0.1x` to `10.0x`).
  - Magnetic frame snapping to grid intervals and edit markers.
- **🌐 Universal Editorial Interchange — ASWF OpenTimelineIO, CMX 3600 EDL, FCPXML & Premiere (`src/export/`)**:
  - **OpenTimelineIO (`.otio`)**: Full `Timeline.1`, `Stack.1`, `Track.1`, `Clip.2`, `Gap.1`, `Marker.2`, `RationalTime.1`, `TimeRange.1`, and `TimeTransform.1` serialization (`src/export/otio.rs`).
  - **SMPTE CMX 3600 EDL (`.edl`)**: Drop-frame (`;`) and non-drop-frame (`:`) SMPTE timecode generator across `23.976`, `24`, `25`, `29.97`, `59.94`, and `60` fps.
  - **Final Cut Pro XML (`.fcpxml` v1.9)**, **Adobe Premiere Pro XML (`<xmeml version="4">`)**, and **DaVinci Resolve Edit Index (`.csv`)**.
- **📈 Cubic Bezier & Damped Spring Keyframe Solver (`src/engine/keyframe_eval.rs`)**:
  - Continuous sub-frame evaluation for Position (`x`, `y`), Scale (`width`, `height`), `rotation`, `opacity`, and `volume`.
  - **Linear**, **Ease-In**, **Ease-Out**, **Ease-InOut**, **Cubic Bezier** (16-step bisection root solver), and **Spring** (underdamped/critically damped harmonic oscillator physics).
- **🎨 WebGPU / Shader Effects & Blend Modes (`src/engine/effects_pipeline.rs`)**:
  - Brightness, Contrast, Saturation, Hue Shift, Sepia, Grayscale, Invert, Vignette, Gaussian Blur, and presets (`Vintage`, `Noir`, `Cold`, `Warm`) plus 12 compositing blend modes.

---

## 📊 Performance Summary

| Workload | Reference (TS / Python OTIO) | Rust `kubeezcut-rs` | Speedup |
| :--- | ---: | ---: | ---: |
| **Multi-Track Split, Trim & Ripple Delete (1k Clips)** | `18.60 ms` | **`0.34 ms`** | **54.7x** |
| **Cubic Bezier & Spring Keyframe Solver (10k Frames)** | `24.10 ms` | **`0.49 ms`** | **49.2x** |
| **ASWF OpenTimelineIO (`.otio`) `Timeline.1` Export** | `41.50 ms` | **`0.88 ms`** | **47.2x** |
| **SMPTE CMX 3600 EDL & FCPXML Export (500 Cuts)** | `29.80 ms` | **`0.61 ms`** | **48.9x** |
| **Total End-to-End Pipeline** | **`123.40 ms`** | **`2.51 ms`** | **49.2x** |

---

## 💻 CLI Usage

```bash
# 1. Create a 4K 60fps project
kubeezcut new --name "Feature Film" --width 3840 --height 2160 --fps 60.0 --output project.kcut

# 2. Export to OpenTimelineIO (.otio), CMX 3600 EDL (.edl), FCPXML, Premiere XML, or DaVinci Resolve CSV
kubeezcut export project.kcut --format otio
kubeezcut export project.kcut --format edl
kubeezcut export project.kcut --format fcpxml
kubeezcut export project.kcut --format premiere
kubeezcut export project.kcut --format resolve-csv

# 3. Validate timeline integrity or compile headless FFmpeg render
kubeezcut validate project.kcut
kubeezcut render project.kcut --dry-run
```

---

## 📜 License

Licensed under the [MIT License](LICENSE).
