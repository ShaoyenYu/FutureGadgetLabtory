use crate::components::*;
use crate::constants::*;
use crate::pixel_art::PixelAssets;
use bevy::prelude::*;
use rand::prelude::random;

/// Where the arena sits on screen. Tiles are kept square and the whole board is
/// centred in the space below the HUD, so pixel art never gets stretched.
pub struct Playfield {
    pub tile: f32,
    /// Bottom-left corner of tile (0, 0), in world space.
    pub origin: Vec2,
    pub center_y: f32,
}

pub fn playfield(window: &Window) -> Playfield {
    let field_width = (window.width() - ARENA_MARGIN * 2.0).max(1.0);
    let field_height = (window.height() - HUD_HEIGHT - ARENA_MARGIN * 2.0).max(1.0);
    let tile = (field_width / ARENA_WIDTH as f32).min(field_height / ARENA_HEIGHT as f32);
    let arena_width = tile * ARENA_WIDTH as f32;
    let arena_height = tile * ARENA_HEIGHT as f32;

    // The HUD eats the top of the window, so the usable band is centred half a
    // HUD below the middle of the screen.
    let center_y = -HUD_HEIGHT / 2.0;

    Playfield {
        tile,
        origin: Vec2::new(-arena_width / 2.0, center_y - arena_height / 2.0),
        center_y,
    }
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

pub fn spawn_arena_grid(mut commands: Commands, assets: Res<PixelAssets>) {
    // Two stacked frames give the board a soft double border.
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: col(COL_FRAME_DARK),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, -0.2),
            ..default()
        },
        ArenaFrame { padding: 18.0 },
    ));
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: col(COL_FRAME),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, -0.1),
            ..default()
        },
        ArenaFrame { padding: 10.0 },
    ));

    for x in 0..ARENA_WIDTH {
        for y in 0..ARENA_HEIGHT {
            let color = if (x + y) % 2 == 0 {
                col(COL_TILE_A)
            } else {
                col(COL_TILE_B)
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

    // A scattering of little flowers so the floor is not a flat checkerboard.
    for _ in 0..18 {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::ONE),
                    ..default()
                },
                texture: assets.flower.clone(),
                transform: Transform::from_xyz(0.0, 0.0, 0.05),
                ..default()
            },
            Position {
                x: (random::<f32>() * ARENA_WIDTH as f32) as i32,
                y: (random::<f32>() * ARENA_HEIGHT as f32) as i32,
            },
            Size::square(1.0),
        ));
    }
}

pub fn size_scaling(windows: Query<&Window>, mut q: Query<(&Size, &mut Transform)>) {
    let Ok(window) = windows.get_single() else {
        return;
    };
    let field = playfield(window);
    for (sprite_size, mut transform) in q.iter_mut() {
        transform.scale = Vec3::new(
            sprite_size.width * field.tile,
            sprite_size.height * field.tile,
            1.0,
        );
    }
}

pub fn position_translation(windows: Query<&Window>, mut q: Query<(&Position, &mut Transform)>) {
    let Ok(window) = windows.get_single() else {
        return;
    };
    let field = playfield(window);
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            field.origin.x + (pos.x as f32 + 0.5) * field.tile,
            field.origin.y + (pos.y as f32 + 0.5) * field.tile,
            transform.translation.z,
        );
    }
}

pub fn update_arena_frame(
    windows: Query<&Window>,
    mut q: Query<(&ArenaFrame, &mut Sprite, &mut Transform)>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };
    let field = playfield(window);
    for (frame, mut sprite, mut transform) in q.iter_mut() {
        sprite.custom_size = Some(Vec2::new(
            ARENA_WIDTH as f32 * field.tile + frame.padding * 2.0,
            ARENA_HEIGHT as f32 * field.tile + frame.padding * 2.0,
        ));
        transform.translation.x = 0.0;
        transform.translation.y = field.center_y;
    }
}

/// Breathing animation. Runs after `size_scaling`, which owns the base scale.
pub fn animate_pulse(time: Res<Time>, mut q: Query<(&Pulse, &mut Transform)>) {
    let elapsed = time.elapsed_seconds();
    for (pulse, mut transform) in q.iter_mut() {
        let factor = 1.0 + pulse.amplitude * (elapsed * pulse.speed + pulse.phase).sin();
        transform.scale.x *= factor;
        transform.scale.y *= factor;
    }
}
