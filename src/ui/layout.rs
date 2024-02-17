use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Debug, Copy, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum HarmonyLayout {
    // [ ][ ]
    // [ ][ ]
    #[default]
    Square,
    // [  ]
    // [  ]
    // [  ]
    // [  ]
    Stacked,
    // ________
    // ||||||||
    // ||||||||
    // --------
    Line,
    Gradient,
}

impl AsRef<str> for HarmonyLayout {
    fn as_ref(&self) -> &str {
        match self {
            Self::Square => "square",
            Self::Stacked => "stacked",
            Self::Line => "line",
            Self::Gradient => "gradient",
        }
    }
}
