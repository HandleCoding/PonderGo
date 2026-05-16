use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Stone {
    Black,
    White,
    Empty,
}

impl Stone {
    pub fn opposite(&self) -> Stone {
        match self {
            Stone::Black => Stone::White,
            Stone::White => Stone::Black,
            Stone::Empty => Stone::Empty,
        }
    }

    pub fn is_black(&self) -> bool {
        matches!(self, Stone::Black)
    }

    pub fn is_white(&self) -> bool {
        matches!(self, Stone::White)
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, Stone::Empty)
    }
}
