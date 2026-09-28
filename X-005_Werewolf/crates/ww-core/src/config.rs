use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::role::{Faction, Role};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelfSave {
    Never,
    FirstNightOnly,
    Always,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WinRule {
    /// 屠边: wolves win once all gods or all villagers are dead.
    KillSide,
    /// 屠城: wolves win once every good player is dead.
    KillAll,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleOptions {
    pub sheriff: bool,
    pub witch_self_save: SelfSave,
    pub win_rule: WinRule,
    /// Players killed on the first night give last words.
    pub first_night_last_words: bool,
    /// Players shot by the hunter give last words.
    pub shot_last_words: bool,
}

impl Default for RuleOptions {
    fn default() -> Self {
        Self {
            sheriff: true,
            witch_self_save: SelfSave::FirstNightOnly,
            win_rule: WinRule::KillSide,
            first_night_last_words: true,
            shot_last_words: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameConfig {
    /// The deck; seat order is decided by the shuffle.
    pub roles: Vec<Role>,
    pub rules: RuleOptions,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ConfigError {
    #[error("player count {0} is outside {min}..={max}", min = GameConfig::MIN_PLAYERS, max = GameConfig::MAX_PLAYERS)]
    PlayerCount(usize),
    #[error("the board needs at least one wolf and one good player")]
    MissingFaction,
    #[error("wolves ({wolves}) must be fewer than good players ({good})")]
    TooManyWolves { wolves: usize, good: usize },
    #[error("at most one {0:?} is allowed")]
    DuplicateGod(Role),
    #[error("the seat layout is not a permutation of the configured deck")]
    RoleMismatch,
}

impl GameConfig {
    pub const MIN_PLAYERS: usize = 6;
    pub const MAX_PLAYERS: usize = 12;

    /// The standard board for a player count (see SPEC §1.3).
    pub fn preset(players: usize) -> Result<Self, ConfigError> {
        use Role::*;
        let (wolves, gods, villagers): (usize, &[Role], usize) = match players {
            6 => (2, &[Seer, Witch], 2),
            7 => (2, &[Seer, Witch, Hunter], 2),
            8 => (3, &[Seer, Witch, Hunter], 2),
            9 => (3, &[Seer, Witch, Hunter], 3),
            10 => (3, &[Seer, Witch, Hunter], 4),
            11 => (4, &[Seer, Witch, Hunter], 4),
            12 => (4, &[Seer, Witch, Hunter], 5),
            n => return Err(ConfigError::PlayerCount(n)),
        };
        let mut roles = vec![Werewolf; wolves];
        roles.extend_from_slice(gods);
        roles.extend(std::iter::repeat_n(Villager, villagers));
        Self::custom(roles, RuleOptions::default())
    }

    pub fn custom(roles: Vec<Role>, rules: RuleOptions) -> Result<Self, ConfigError> {
        let cfg = Self { roles, rules };
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn players(&self) -> usize {
        self.roles.len()
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        let n = self.roles.len();
        if !(Self::MIN_PLAYERS..=Self::MAX_PLAYERS).contains(&n) {
            return Err(ConfigError::PlayerCount(n));
        }
        let wolves = self.roles.iter().filter(|r| r.faction() == Faction::Wolf).count();
        let good = n - wolves;
        if wolves == 0 || good == 0 {
            return Err(ConfigError::MissingFaction);
        }
        if wolves >= good {
            return Err(ConfigError::TooManyWolves { wolves, good });
        }
        for god in [Role::Seer, Role::Witch, Role::Hunter] {
            if self.roles.iter().filter(|r| **r == god).count() > 1 {
                return Err(ConfigError::DuplicateGod(god));
            }
        }
        Ok(())
    }
}
