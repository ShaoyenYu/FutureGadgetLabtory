use bevy::prelude::*;

#[derive(States, Default, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    Loading,
    MainMenu,
    BaseCamp,
    RunSpawning,
    InRun,
    Extraction,
    GameOver,
}
