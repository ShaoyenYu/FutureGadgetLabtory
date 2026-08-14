pub mod animation;
pub mod camera;
pub mod pixel_generator;
pub mod tilemap;

use animation::*;
use bevy::prelude::*;
use camera::*;
use pixel_generator::*;
use soulforge_core::states::AppState;
use tilemap::*;

pub struct SoulforgeRenderPlugin;

impl Plugin for SoulforgeRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PixelAssets>()
            .init_resource::<CameraScreenShake>()
            .add_systems(Startup, (setup_camera_system, generate_all_pixel_assets))
            .add_systems(
                Update,
                (
                    animation_update_system,
                    camera_follow_system,
                    update_dungeon_tiles_system,
                )
                    .run_if(in_state(AppState::InRun).or_else(in_state(AppState::Extraction))),
            );
    }
}

fn setup_camera_system(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 999.0),
            ..default()
        },
        MainGameCamera,
    ));
}
