# `kubeezcut-rs` 🦀🎬

[![Rust](https://img.shields.io/badge/Rust-1.80+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/Tests-27%20Passing-brightgreen.svg)]()

A high-performance, native pure **Rust** port of [KubeezMedia/KubeezCut](https://github.com/KubeezMedia/KubeezCut) — a modern non-linear video editing (NLE) timeline architecture, WebGPU/shader effects pipeline, keyframe Bezier animation engine, and headless render export.

---

## ⚡ Highlights & Architecture

- **🚀 Native Compiled Engine**: Pure Rust NLE timeline model with sub-millisecond timeline operations, zero garbage collection pauses, and static binary deployment.
- **🎞️ Multi-Track Timeline Engine**:
  - Video and Audio tracks with volume control, mute, and solo.
  - Non-destructive clip editing: **Split at playhead**, **Ripple delete**, **Trim In / Trim Out**, and **Slip / Slide editing**.
  - Magnetic snapping to grid and edit markers.
  - Multi-source trimming and variable playback speed multipliers (`0.1x` to `10.0x`).
- **📈 Advanced Keyframe Bezier & Spring Physics**:
  - Continuous property evaluation across any arbitrary frame: Position (`x`, `y`), Scale (`width`, `height`), `rotation`, `opacity`, and `volume`.
  - Interpolation curves: **Linear**, **Ease-In**, **Ease-Out**, **Ease-InOut**, **Cubic Bezier** (bisection search solver), and **Spring** (damped harmonic oscillator physics).
- **🎨 WebGPU / Shader Effects Pipeline**:
  - Complete effect registry: Brightness, Contrast, Saturation, Hue Shift, Sepia, Grayscale, Invert, Vignette, Blur, and Chroma Key.
  - One-click presets: `Vintage`, `Noir`, `Cold`, and `Warm`.
  - Layer blend modes: `Normal`, `Multiply`, `Screen`, `Overlay`, `Darken`, `Lighten`, `ColorDodge`, `ColorBurn`, `HardLight`, `SoftLight`, `Difference`, `Exclusion`.
- **🔊 Audio Mixing & Smooth Envelopes**:
  - Decibel-to-linear amplitude calculations.
  - Configurable Audio Fade-In and Fade-Out curves.
- **📦 NLE Project Interchange & Headless Render**:
  - Full roundtrip serialization for native `.kcut` / `.json` project schemas (v6 schema parity).
  - **Final Cut Pro XML** (`.fcpxml` v1.9 / v1.10) export.
  - **Adobe Premiere Pro XML** (`<xmeml version="4">`) export.
  - Headless FFmpeg render orchestrator with hardware acceleration (`h264_videotoolbox` on macOS, `h264_nvenc` on Linux).

---

## 🚀 Installation & Build

```bash
git clone https://github.com/bhubbard/kubeezcut-rs.git
cd kubeezcut-rs
cargo build --release
```

Binary will be available at `./target/release/kubeezcut`.

---

## 💻 CLI Usage

### 1. Create a New Project
```bash
# Create a 4K 60fps project
kubeezcut new --name "Feature Film" --width 3840 --height 2160 --fps 60.0 --output project.kcut
```

### 2. Inspect a Project
```bash
kubeezcut inspect project.kcut
```

### 3. Validate Timeline Integrity
```bash
kubeezcut validate project.kcut
```

### 4. Export to Final Cut Pro or Premiere Pro
```bash
# Export to Final Cut Pro XML
kubeezcut export project.kcut --format fcpxml

# Export to Adobe Premiere Pro XML
kubeezcut export project.kcut --format premiere
```

### 5. Headless Video Render
```bash
# Render to MP4 with hardware acceleration
kubeezcut render project.kcut --output final_cut.mp4

# Print dry-run FFmpeg filtergraph command
kubeezcut render project.kcut --dry-run
```

---

## 🧪 Upstream Parity & Test Suite

Includes 27 automated unit and integration tests covering:
- Multi-track timeline sequencing and total duration calculations
- Split clip operations preserving source boundaries and speed
- Ripple delete gap consolidation and downstream track item shifting
- Trim In / Trim Out boundary adjustments
- Slip and slide non-destructive media offsets
- Magnetic snapping to edit markers
- Cubic Bezier curve and spring physics keyframe animation evaluation
- Audio decibel gain calculations and fade-in / fade-out envelopes
- GPU shader effects compilation and blend modes
- Final Cut Pro and Adobe Premiere Pro XML exports
- Headless hardware-accelerated render pipeline compilation

Run test suite:
```bash
cargo test
```

---

## 📜 License

Licensed under the [MIT License](LICENSE). Based on the architecture and concepts of [KubeezMedia/KubeezCut](https://github.com/KubeezMedia/KubeezCut).
