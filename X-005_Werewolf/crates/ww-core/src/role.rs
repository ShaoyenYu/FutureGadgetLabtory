use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Role {
    Werewolf,
    Villager,
    Seer,
    Witch,
    Hunter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Faction {
    Wolf,
    Good,
}

/// Win-condition bucket: 屠边 counts gods and villagers separately.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    Wolf,
    God,
    Villager,
}

impl Role {
    pub fn faction(self) -> Faction {
        match self {
            Role::Werewolf => Faction::Wolf,
            _ => Faction::Good,
        }
    }

    pub fn category(self) -> Category {
        match self {
            Role::Werewolf => Category::Wolf,
            Role::Villager => Category::Villager,
            Role::Seer | Role::Witch | Role::Hunter => Category::God,
        }
    }

    pub fn name_zh(self) -> &'static str {
        match self {
            Role::Werewolf => "狼人",
            Role::Villager => "村民",
            Role::Seer => "预言家",
            Role::Witch => "女巫",
            Role::Hunter => "猎人",
        }
    }
}
