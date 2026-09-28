//! Werewolf rules engine.
//!
//! Pure and deterministic: no IO, no async. The [`Game`] is the only holder of the
//! god view; everything a player may learn must pass through [`Game::observe`].

mod config;
mod decision;
mod event;
mod game;
mod role;
mod seat;
pub mod sim;
mod visibility;

pub use config::{ConfigError, GameConfig, RuleOptions, SelfSave, WinRule};
pub use decision::{Decision, DecisionRequest, Direction, SpeechKind, VoteKind, WitchAction};
pub use event::{DeathCause, Event, EventKind, NightStep, Phase, Step, Visibility};
pub use game::{Game, RuleError};
pub use role::{Category, Faction, Role};
pub use seat::SeatId;
pub use visibility::Observation;
