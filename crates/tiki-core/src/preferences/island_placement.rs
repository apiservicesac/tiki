use serde::{Deserialize, Serialize};

use crate::error::{TikiError, TikiResult};

/// Screen edge the island hangs from.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IslandPlacement {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
}

impl IslandPlacement {
    pub const ALL: [IslandPlacement; 4] = [
        IslandPlacement::Top,
        IslandPlacement::Bottom,
        IslandPlacement::Left,
        IslandPlacement::Right,
    ];

    /// Stable key used in storage and in translation keys (`placement.<key>`).
    pub fn key(self) -> &'static str {
        match self {
            IslandPlacement::Top => "top",
            IslandPlacement::Bottom => "bottom",
            IslandPlacement::Left => "left",
            IslandPlacement::Right => "right",
        }
    }

    pub fn from_key(key: &str) -> TikiResult<Self> {
        Self::ALL
            .into_iter()
            .find(|placement| placement.key() == key)
            .ok_or_else(|| TikiError::UnknownValue {
                kind: "island placement",
                value: key.to_owned(),
            })
    }
}
