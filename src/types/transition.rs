use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TransitionKind {
    FadeToBlack,
    FadeToWhite,
    CrossDissolve,
    WipeLeft,
    WipeRight,
    WipeUp,
    WipeDown,
    SlideLeft,
    SlideRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TransitionAlignment {
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transition {
    pub id: String,
    pub kind: TransitionKind,
    pub from_item_id: String,
    pub to_item_id: String,
    pub duration_in_frames: i64,
    pub alignment: TransitionAlignment,
}

impl Transition {
    pub fn new(
        id: impl Into<String>,
        kind: TransitionKind,
        from_item_id: impl Into<String>,
        to_item_id: impl Into<String>,
        duration_in_frames: i64,
    ) -> Self {
        Self {
            id: id.into(),
            kind,
            from_item_id: from_item_id.into(),
            to_item_id: to_item_id.into(),
            duration_in_frames,
            alignment: TransitionAlignment::Center,
        }
    }
}
