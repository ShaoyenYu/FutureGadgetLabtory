use std::fmt;

use serde::{Deserialize, Serialize};

/// 1-based seat number, as players call it ("3号").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SeatId(pub u8);

impl SeatId {
    pub fn index(self) -> usize {
        self.0 as usize - 1
    }

    pub fn from_index(i: usize) -> Self {
        SeatId(i as u8 + 1)
    }
}

impl fmt::Display for SeatId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}号", self.0)
    }
}
