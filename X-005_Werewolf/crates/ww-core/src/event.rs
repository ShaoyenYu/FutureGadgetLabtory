use serde::{Deserialize, Serialize};

use crate::decision::{Direction, SpeechKind, VoteKind};
use crate::role::{Faction, Role};
use crate::seat::SeatId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Setup,
    Night(u32),
    Day(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NightStep {
    WolfKill,
    Witch,
    Seer,
}

/// Every step is announced publicly (the moderator calls it out loud), even when
/// nobody can act in it, so that step boundaries never leak who is alive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Step {
    Setup,
    Night(NightStep),
    SheriffSignup,
    SheriffSpeech,
    SheriffWithdraw,
    SheriffVote,
    SheriffPkSpeech,
    SheriffPkVote,
    Announce,
    DeathResolution,
    SpeakOrder,
    Speech,
    ExileVote,
    ExilePkSpeech,
    ExilePkVote,
    GameOver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Team(Faction),
    Private(SeatId),
    GodOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeathCause {
    WolfKill,
    Poison,
    Exile,
    HunterShot,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub seq: u64,
    pub phase: Phase,
    pub vis: Visibility,
    pub kind: EventKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EventKind {
    RoleAssigned { seat: SeatId, role: Role },
    WolfTeammates { wolves: Vec<SeatId> },
    PhaseStarted { phase: Phase },
    StepStarted { step: Step },
    WolfChat { from: SeatId, text: String },
    WolfVoteCast { from: SeatId, target: Option<SeatId> },
    WolfTarget { target: Option<SeatId> },
    WitchInfo { victim: Option<SeatId> },
    WitchSaved { target: SeatId },
    WitchPoisoned { target: SeatId },
    SeerResult { target: SeatId, is_wolf: bool },
    NightResolved { deaths: Vec<(SeatId, DeathCause)> },
    SheriffCandidates { seats: Vec<SeatId> },
    SheriffWithdrew { seat: SeatId },
    SheriffElected { seat: Option<SeatId> },
    /// Causes are never announced.
    DeathsAnnounced { seats: Vec<SeatId> },
    Speech { seat: SeatId, kind: SpeechKind, text: String },
    /// Votes are revealed together once the vote closes. Weight in half votes.
    VoteCast { kind: VoteKind, voter: SeatId, target: Option<SeatId>, weight_x2: u8 },
    VoteResult { kind: VoteKind, winner: Option<SeatId>, tied: Vec<SeatId> },
    HunterShot { hunter: SeatId, target: SeatId },
    BadgeTransferred { from: SeatId, to: Option<SeatId> },
    SpeakOrderChosen { first: SeatId, dir: Direction },
    Exiled { seat: Option<SeatId> },
    GameOver { winner: Faction, roles: Vec<(SeatId, Role)> },
}
