use serde::{Deserialize, Serialize};

use crate::seat::SeatId;

/// Speaking direction; clockwise means increasing seat numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Clockwise,
    CounterClockwise,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpeechKind {
    SheriffCampaign,
    SheriffPk,
    Normal,
    /// The sheriff speaks last and recommends whom to exile (归票).
    SheriffSummary,
    ExilePk,
    LastWords,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoteKind {
    Sheriff,
    SheriffPk,
    Exile,
    ExilePk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WitchAction {
    Save,
    Poison(SeatId),
    Pass,
}

/// What the engine asks one seat to decide. Only ever sent to that seat.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecisionRequest {
    WolfKill { candidates: Vec<SeatId>, teammates: Vec<SeatId> },
    WitchAct {
        /// Tonight's victim; only revealed while the antidote is unused.
        victim: Option<SeatId>,
        can_save: bool,
        can_poison: bool,
        poison_candidates: Vec<SeatId>,
    },
    SeerCheck { candidates: Vec<SeatId> },
    SheriffSignup,
    /// Last chance to withdraw after all campaign speeches (退水).
    SheriffWithdrawWindow,
    Speak { kind: SpeechKind },
    Vote { kind: VoteKind, candidates: Vec<SeatId> },
    HunterShoot { candidates: Vec<SeatId> },
    BadgeTransfer { candidates: Vec<SeatId> },
    ChooseSpeakOrder,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decision {
    WolfKill(Option<SeatId>),
    /// Out-of-band: any living wolf, any time during the wolf step.
    WolfChat(String),
    Witch(WitchAction),
    SeerCheck(Option<SeatId>),
    SheriffSignup(bool),
    /// Out-of-band during campaign speeches, or in the withdraw window.
    SheriffWithdraw,
    Stay,
    Speak(String),
    Vote(Option<SeatId>),
    HunterShoot(Option<SeatId>),
    /// `None` tears up the badge.
    BadgeTransfer(Option<SeatId>),
    SpeakOrder(Direction),
}

impl DecisionRequest {
    /// What happens when the seat runs out of time.
    pub fn default_decision(&self) -> Decision {
        match self {
            DecisionRequest::WolfKill { .. } => Decision::WolfKill(None),
            DecisionRequest::WitchAct { .. } => Decision::Witch(WitchAction::Pass),
            DecisionRequest::SeerCheck { .. } => Decision::SeerCheck(None),
            DecisionRequest::SheriffSignup => Decision::SheriffSignup(false),
            DecisionRequest::SheriffWithdrawWindow => Decision::Stay,
            DecisionRequest::Speak { .. } => Decision::Speak(String::new()),
            DecisionRequest::Vote { .. } => Decision::Vote(None),
            DecisionRequest::HunterShoot { .. } => Decision::HunterShoot(None),
            DecisionRequest::BadgeTransfer { .. } => Decision::BadgeTransfer(None),
            DecisionRequest::ChooseSpeakOrder => Decision::SpeakOrder(Direction::Clockwise),
        }
    }
}
