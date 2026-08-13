use crate::components::*;
use crate::constants::*;
use bevy::prelude::*;

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

pub fn spawn_arena_grid(mut commands: Commands) {
    for x in 0..ARENA_WIDTH {
        for y in 0..ARENA_HEIGHT {
            let is_light = (x + y) % 2 == 0;
            let color = if is_light {
                Color::srgb(0.08, 0.08, 0.08)
            } else {
                Color::srgb(0.05, 0.05, 0.05)
            };

            commands.spawn((
                SpriteBundle {
                    sprite: Sprite { color, ..default() },
                    transform: Transform::from_xyz(0.0, 0.0, 0.0),
                    ..default()
                },
                Position {
                    x: x as i32,
                    y: y as i32,
                },
                Size::square(1.0),
            ));
        }
    }
}

pub fn size_scaling(windows: Query<&Window>, mut q: Query<(&Size, &mut Transform)>) {
    let window = windows.single();
    for (sprite_size, mut transform) in q.iter_mut() {
        transform.scale = Vec3::new(
            sprite_size.width / ARENA_WIDTH as f32 * window.width(),
            sprite_size.height / ARENA_HEIGHT as f32 * window.height(),
            1.0,
        );
    }
}

pub fn position_translation(windows: Query<&Window>, mut q: Query<(&Position, &mut Transform)>) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
    }
    let window = windows.single();
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width(), ARENA_WIDTH as f32),
            convert(pos.y as f32, window.height(), ARENA_HEIGHT as f32),
            transform.translation.z,
        );
    }
}
