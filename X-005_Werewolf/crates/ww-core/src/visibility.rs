use serde::Serialize;

use crate::event::{Event, EventKind, Phase};

/// What one seat is allowed to know about one event.
///
/// Fields are private and the only constructor is crate-internal, so outside code
/// can only obtain an `Observation` through [`crate::Game::observe`], which applies
/// the visibility filter. Handing a raw [`Event`] to a seat is a type error.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Observation {
    seq: u64,
    phase: Phase,
    kind: EventKind,
}

impl Observation {
    pub(crate) fn from_event(e: &Event) -> Self {
        Self { seq: e.seq, phase: e.phase, kind: e.kind.clone() }
    }

    pub fn seq(&self) -> u64 {
        self.seq
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn kind(&self) -> &EventKind {
        &self.kind
    }
}
