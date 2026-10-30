use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AnimatableProperty {
    X,
    Y,
    Width,
    Height,
    Rotation,
    Opacity,
    CornerRadius,
    Volume,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EasingType {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    CubicBezier,
    Spring,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BezierControlPoints {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

impl Default for BezierControlPoints {
    fn default() -> Self {
        Self {
            x1: 0.25,
            y1: 0.1,
            x2: 0.25,
            y2: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpringParameters {
    pub tension: f64, // e.g. 170.0
    pub friction: f64, // e.g. 26.0
    pub mass: f64,     // e.g. 1.0
}

impl Default for SpringParameters {
    fn default() -> Self {
        Self {
            tension: 170.0,
            friction: 26.0,
            mass: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EasingConfig {
    pub easing: EasingType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bezier: Option<BezierControlPoints>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spring: Option<SpringParameters>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Keyframe {
    pub id: String,
    pub frame: i64,
    pub value: f64,
    pub easing: EasingType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub easing_config: Option<EasingConfig>,
}

impl Keyframe {
    pub fn new(id: impl Into<String>, frame: i64, value: f64, easing: EasingType) -> Self {
        Self {
            id: id.into(),
            frame,
            value,
            easing,
            easing_config: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PropertyKeyframes {
    pub property: AnimatableProperty,
    pub keyframes: Vec<Keyframe>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemKeyframes {
    pub item_id: String,
    pub properties: Vec<PropertyKeyframes>,
}
