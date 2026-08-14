pub mod components;
pub mod constants;
pub mod pixel_art;
pub mod systems;

use bevy::{prelude::*, time::common_conditions::on_timer};
use std::time::Duration;

use components::*;
use constants::*;
use pixel_art::setup_pixel_assets;
use systems::environment::*;
use systems::render::*;
use systems::snake::*;
use systems::ui::*;

/// Builds and configures the core DuoSnake Bevy application.
pub fn create_app() -> App {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Snake Duo - 2 Players".into(),
                    resolution: (1280.0_f32, 720.0_f32).into(),
                    canvas: Some("#bevy".into()),
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: true,
                    ..default()
                }),
                ..default()
            })
            // Pixel art must not be smoothed when scaled up to tile size.
            .set(ImagePlugin::default_nearest()),
    )
    .init_state::<GameState>()
    .insert_resource(ClearColor(col(COL_BG)))
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
        // Everything below draws with the baked textures, so they go first.
        (
            setup_pixel_assets,
            (
                setup_camera,
                setup_menus,
                spawn_ui,
                spawn_arena_grid,
                spawn_snakes,
            ),
        )
            .chain(),
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
            handle_player_death,
            update_respawn_timers,
            restart_game,
            update_snake_visuals,
            animate_blink,
            animate_pulse.after(size_scaling),
        )
            .run_if(in_state(GameState::Playing)),
    )
    // Layout and HUD keep running while paused so a resize is never stale.
    .add_systems(
        Update,
        (
            (position_translation, size_scaling),
            update_arena_frame,
            update_hud,
            toggle_pause,
            ui_interaction,
        ),
    )
    .add_systems(OnEnter(GameState::Paused), show_pause_menu)
    .add_systems(OnExit(GameState::Paused), hide_pause_menu)
    .add_systems(OnEnter(GameState::Settings), show_settings_menu)
    .add_systems(OnExit(GameState::Settings), hide_settings_menu);

    app
}

/// Runs the DuoSnake application.
pub fn run() {
    create_app().run();
}
