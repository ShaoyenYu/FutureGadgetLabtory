use bevy::prelude::*;

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Player {
    One,
    Two,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}
impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}

#[derive(Component)]
pub struct SnakeHead {
    pub direction: SnakeDirection,
}

#[derive(Component)]
pub struct SnakeSegment;

#[derive(Resource, Default)]
pub struct SnakeSegments {
    pub player1: Vec<Entity>,
    pub player2: Vec<Entity>,
}

#[derive(Component)]
pub struct Food;

#[derive(Component)]
pub struct Trap {
    pub timer: Timer,
}

#[derive(Component)]
pub struct TrapTile(pub Entity);

#[derive(Component)]
pub struct TrapText(pub Entity);

#[derive(Resource, Default)]
pub struct Scores {
    pub player1: u32,
    pub player2: u32,
}

#[derive(Component)]
pub struct ScoreText(pub Player);

#[derive(Resource)]
pub struct PlayerHP {
    pub player1: u32,
    pub player2: u32,
}
impl Default for PlayerHP {
    fn default() -> Self {
        Self {
            player1: 5,
            player2: 5,
        }
    }
}

#[derive(Resource)]
pub struct GameSettings {
    pub initial_hp: u32,
}
impl Default for GameSettings {
    fn default() -> Self {
        Self { initial_hp: 5 }
    }
}

#[derive(Component)]
pub enum UIAction {
    Resume,
    Settings,
    Back,
    IncreaseHP,
    DecreaseHP,
    Restart,
}

#[derive(Component)]
pub struct HPText;

#[derive(Component)]
pub struct PauseMenuUI;

#[derive(Component)]
pub struct SettingsMenuUI;

#[derive(Resource, Default)]
pub struct PlayerStates {
    pub p1_respawn_timer: Option<Timer>,
    pub p2_respawn_timer: Option<Timer>,
}

#[derive(Component)]
pub struct RespawnText(pub Player);

#[derive(Component)]
pub struct PausedUI;

#[derive(Event)]
pub struct RestartGameEvent;

#[derive(PartialEq, Copy, Clone)]
pub enum SnakeDirection {
    Left,
    Up,
    Right,
    Down,
}

impl SnakeDirection {
    pub fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}

#[derive(Event)]
pub struct PlayerDiedEvent(pub Player);

#[derive(Event)]
pub struct GrowthEvent(pub Player);

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    #[default]
    Playing,
    Paused,
    Settings,
}
