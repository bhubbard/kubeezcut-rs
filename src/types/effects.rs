use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
}

impl Default for BlendMode {
    fn default() -> Self {
        BlendMode::Normal
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GpuEffect {
    pub gpu_effect_type: String, // e.g. "gpu-brightness", "gpu-contrast", "gpu-sepia"
    pub params: HashMap<String, f64>,
}

impl GpuEffect {
    pub fn new(effect_type: impl Into<String>) -> Self {
        Self {
            gpu_effect_type: effect_type.into(),
            params: HashMap::new(),
        }
    }

    pub fn with_param(mut self, key: impl Into<String>, val: f64) -> Self {
        self.params.insert(key.into(), val);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemEffect {
    pub id: String,
    pub effect: GpuEffect,
    pub enabled: bool,
}

impl ItemEffect {
    pub fn new(id: impl Into<String>, effect: GpuEffect) -> Self {
        Self {
            id: id.into(),
            effect,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectPreset {
    pub id: String,
    pub name: String,
    pub effects: Vec<GpuEffect>,
}

impl EffectPreset {
    pub fn vintage() -> Self {
        Self {
            id: "vintage".to_string(),
            name: "Vintage".to_string(),
            effects: vec![
                GpuEffect::new("gpu-sepia").with_param("amount", 0.4),
                GpuEffect::new("gpu-contrast").with_param("amount", 1.1),
                GpuEffect::new("gpu-brightness").with_param("amount", -0.1),
            ],
        }
    }

    pub fn noir() -> Self {
        Self {
            id: "noir".to_string(),
            name: "Noir".to_string(),
            effects: vec![
                GpuEffect::new("gpu-grayscale").with_param("amount", 1.0),
                GpuEffect::new("gpu-contrast").with_param("amount", 1.3),
            ],
        }
    }

    pub fn cold() -> Self {
        Self {
            id: "cold".to_string(),
            name: "Cold".to_string(),
            effects: vec![
                GpuEffect::new("gpu-hue-shift").with_param("shift", 0.5),
                GpuEffect::new("gpu-saturation").with_param("amount", 0.8),
            ],
        }
    }

    pub fn warm() -> Self {
        Self {
            id: "warm".to_string(),
            name: "Warm".to_string(),
            effects: vec![
                GpuEffect::new("gpu-sepia").with_param("amount", 0.2),
                GpuEffect::new("gpu-saturation").with_param("amount", 1.2),
            ],
        }
    }
}
