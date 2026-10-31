//! # MatchCut & AAF Interchange (`matchcut-rs` + AAF Structure Mapping)
//!
//! Pure Rust implementation of `Netflix/matchcut` visual continuity & match-cut transition
//! discovery engine + AAF (Advanced Authoring Format) CompositionMob/MasterMob structure mapping.

use serde::{Deserialize, Serialize};

/// Visual + motion representation of a timeline shot for `Netflix/matchcut` discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShotEmbedding {
    pub shot_id: String,
    pub start_frame: u64,
    pub end_frame: u64,
    /// Visual feature vector (e.g. SigLIP / CLIP / ResNet-50 embedding).
    pub visual_features: Vec<f32>,
    /// Normalized 2D optical flow / camera motion vector `(vx, vy)`.
    pub motion_vector: (f32, f32),
    /// Normalized composition subject centroid `(cx, cy)` in `[0.0, 1.0]`.
    pub composition_centroid: (f32, f32),
}

/// Metric learning projection head ported from `matchcut/models.py`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricProjectionHead {
    pub input_dim: usize,
    pub output_dim: usize,
    pub weights: Vec<f32>,
}

impl MetricProjectionHead {
    /// Creates a deterministic orthogonal-like projection head for embedding compression.
    pub fn identity_like(dim: usize) -> Self {
        let mut weights = vec![0.0f32; dim * dim];
        for i in 0..dim {
            weights[i * dim + i] = 1.0;
        }
        Self {
            input_dim: dim,
            output_dim: dim,
            weights,
        }
    }

    /// Projects and L2-normalizes a visual feature vector.
    pub fn project_and_normalize(&self, features: &[f32]) -> Vec<f32> {
        let n = self.input_dim.min(features.len());
        let mut out = vec![0.0f32; self.output_dim];
        for (row, out_val) in out.iter_mut().enumerate() {
            let row_offset = row * self.input_dim;
            let mut sum = 0.0f32;
            for col in 0..n {
                sum += self.weights[row_offset + col] * features[col];
            }
            *out_val = sum;
        }
        let norm = out.iter().map(|v| v * v).sum::<f32>().sqrt().max(1e-8);
        for v in &mut out {
            *v /= norm;
        }
        out
    }
}

/// Ranked match-cut transition candidate between two shots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchCutCandidate {
    pub outgoing_shot_id: String,
    pub incoming_shot_id: String,
    pub cut_frame: u64,
    pub cosine_similarity: f32,
    pub motion_alignment: f32,
    pub framing_alignment: f32,
    pub composite_score: f32,
}

/// Algorithmic visual continuity and match-cut discovery engine (`Netflix/matchcut`).
pub struct MatchCutEngine;

impl MatchCutEngine {
    /// Computes cosine similarity between two L2-normalized or unnormalized vectors.
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let len = a.len().min(b.len());
        if len == 0 {
            return 0.0;
        }
        let mut dot = 0.0f32;
        let mut na = 0.0f32;
        let mut nb = 0.0f32;
        for i in 0..len {
            dot += a[i] * b[i];
            na += a[i] * a[i];
            nb += b[i] * b[i];
        }
        let denom = (na.sqrt() * nb.sqrt()).max(1e-8);
        (dot / denom).clamp(-1.0, 1.0)
    }

    /// Discovers and ranks top match-cut transitions across a library of shots.
    pub fn discover_match_cuts(
        shots: &[ShotEmbedding],
        head: Option<&MetricProjectionHead>,
        min_score: f32,
    ) -> Vec<MatchCutCandidate> {
        let mut candidates = Vec::new();
        for (i, a) in shots.iter().enumerate() {
            let emb_a = head
                .map(|h| h.project_and_normalize(&a.visual_features))
                .unwrap_or_else(|| a.visual_features.clone());
            for (j, b) in shots.iter().enumerate() {
                if i == j {
                    continue;
                }
                let emb_b = head
                    .map(|h| h.project_and_normalize(&b.visual_features))
                    .unwrap_or_else(|| b.visual_features.clone());

                let cos = Self::cosine_similarity(&emb_a, &emb_b);
                let motion_dist = ((a.motion_vector.0 - b.motion_vector.0).powi(2)
                    + (a.motion_vector.1 - b.motion_vector.1).powi(2))
                .sqrt();
                let motion_alignment = (1.0 - motion_dist * 0.5).clamp(0.0, 1.0);

                let centroid_dist = ((a.composition_centroid.0 - b.composition_centroid.0).powi(2)
                    + (a.composition_centroid.1 - b.composition_centroid.1).powi(2))
                .sqrt();
                let framing_alignment = (1.0 - centroid_dist).clamp(0.0, 1.0);

                let composite_score =
                    0.55 * cos.max(0.0) + 0.25 * motion_alignment + 0.20 * framing_alignment;
                if composite_score >= min_score {
                    candidates.push(MatchCutCandidate {
                        outgoing_shot_id: a.shot_id.clone(),
                        incoming_shot_id: b.shot_id.clone(),
                        cut_frame: a.end_frame,
                        cosine_similarity: cos,
                        motion_alignment,
                        framing_alignment,
                        composite_score,
                    });
                }
            }
        }
        candidates.sort_by(|x, y| {
            y.composite_score
                .partial_cmp(&x.composite_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        candidates
    }

    /// Exports match-cut candidates as FCPXML `<marker>` annotations.
    pub fn export_fcpxml_markers(candidates: &[MatchCutCandidate], fps: f64) -> String {
        let mut xml = String::new();
        for c in candidates {
            let start_sec = (c.cut_frame as f64) / fps.max(1.0);
            xml.push_str(&format!(
                "<marker start=\"{:.3}s\" duration=\"1/{}s\" value=\"MatchCut: {} -> {} (score={:.3})\"/>\n",
                start_sec,
                fps.round() as u32,
                c.outgoing_shot_id,
                c.incoming_shot_id,
                c.composite_score
            ));
        }
        xml
    }
}

/// AAF (Advanced Authoring Format) CompositionMob / MasterMob / SourceMob structure mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AafCompositionPackage {
    pub composition_mob_id: String,
    pub title: String,
    pub edit_rate_fps: f64,
    pub master_mobs: Vec<String>,
    pub source_mobs: Vec<String>,
}

impl AafCompositionPackage {
    /// Maps a list of shot IDs into an AAF CompositionMob + MasterMob + SourceMob hierarchy.
    pub fn from_shot_ids(title: &str, edit_rate_fps: f64, shot_ids: &[String]) -> Self {
        let master_mobs = shot_ids
            .iter()
            .map(|id| format!("urn:smpte:umid:master.{}", id))
            .collect();
        let source_mobs = shot_ids
            .iter()
            .map(|id| format!("urn:smpte:umid:source.{}", id))
            .collect();
        Self {
            composition_mob_id: format!("urn:smpte:umid:comp.{}", title.to_lowercase().replace(' ', "_")),
            title: title.to_string(),
            edit_rate_fps,
            master_mobs,
            source_mobs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matchcut_discovery_and_fcpxml_markers() {
        let shots = vec![
            ShotEmbedding {
                shot_id: "shot_01".to_string(),
                start_frame: 0,
                end_frame: 120,
                visual_features: vec![0.9, 0.1, 0.0, 0.0],
                motion_vector: (0.2, 0.0),
                composition_centroid: (0.5, 0.5),
            },
            ShotEmbedding {
                shot_id: "shot_02".to_string(),
                start_frame: 120,
                end_frame: 240,
                visual_features: vec![0.88, 0.12, 0.0, 0.0],
                motion_vector: (0.21, 0.0),
                composition_centroid: (0.51, 0.49),
            },
        ];
        let head = MetricProjectionHead::identity_like(4);
        let candidates = MatchCutEngine::discover_match_cuts(&shots, Some(&head), 0.8);
        assert_eq!(candidates.len(), 2);
        assert!(candidates[0].composite_score > 0.95);
        let markers = MatchCutEngine::export_fcpxml_markers(&candidates, 24.0);
        assert!(markers.contains("MatchCut: shot_01 -> shot_02"));

        let aaf = AafCompositionPackage::from_shot_ids("Commercial_Cut", 24.0, &[String::from("shot_01")]);
        assert_eq!(aaf.master_mobs.len(), 1);
    }
}
