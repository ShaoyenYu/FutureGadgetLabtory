mod components;
mod constants;
mod systems;

use bevy::{prelude::*, time::common_conditions::on_timer};
use std::time::Duration;

use components::*;
use systems::environment::*;
use systems::render::*;
use systems::snake::*;
use systems::ui::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Snake - 2 Players".into(),
                resolution: (1920.0_f32, 1080.0_f32).into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .insert_resource(ClearColor(Color::srgb(0.04, 0.04, 0.04)))
        .insert_resource(SnakeSegments::default())
        .insert_resource(Scores::default())
        .insert_resource(PlayerStates::default())
        .insert_resource(GameSettings::default())
        .insert_resource(PlayerHP::default())
        .add_event::<GrowthEvent>()
        .add_event::<PlayerDiedEvent>()
        .add_event::<RestartGameEvent>()
        .add_systems(
            Startup,
            (
                setup_camera,
                setup_menus,
                spawn_ui,
                spawn_arena_grid,
                spawn_snakes,
            ),
        )
        .add_systems(
            Update,
            (
                snake_movement_input.before(snake_movement),
                snake_movement.run_if(on_timer(Duration::from_millis(120))),
                food_spawner.run_if(on_timer(Duration::from_secs(1))),
                trap_spawner.run_if(on_timer(Duration::from_secs(2))),
                update_traps,
                snake_eating,
                snake_growth,
                update_score_ui,
                handle_player_death,
                update_respawn_timers,
                position_translation,
                size_scaling,
                restart_game,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(Update, (toggle_pause, ui_interaction))
        .add_systems(OnEnter(GameState::Paused), show_pause_menu)
        .add_systems(OnExit(GameState::Paused), hide_pause_menu)
        .add_systems(OnEnter(GameState::Settings), show_settings_menu)
        .add_systems(OnExit(GameState::Settings), hide_settings_menu)
        .run();
}
