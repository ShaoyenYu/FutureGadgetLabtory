use crate::components::*;
use crate::constants::*;
use crate::pixel_art::PixelAssets;
use bevy::prelude::*;
use rand::prelude::random;
use std::f32::consts::TAU;

pub fn food_spawner(mut commands: Commands, q_food: Query<&Food>, assets: Res<PixelAssets>) {
    let current_food = q_food.iter().count();
    if current_food == 0 {
        let amount = (random::<f32>() * 7.0) as usize + 1; // 1 to 7
        for _ in 0..amount {
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2::ONE),
                        ..default()
                    },
                    texture: assets.apple.clone(),
                    transform: Transform::from_xyz(0.0, 0.0, 1.5),
                    ..default()
                },
                Food,
                Position {
                    x: (random::<f32>() * ARENA_WIDTH as f32) as i32,
                    y: (random::<f32>() * ARENA_HEIGHT as f32) as i32,
                },
                Size::square(0.88),
                Pulse {
                    amplitude: 0.07,
                    speed: 3.4,
                    phase: random::<f32>() * TAU,
                },
            ));
        }
    }
}

pub fn trap_spawner(mut commands: Commands, traps: Query<&Trap>, assets: Res<PixelAssets>) {
    if random::<f32>() < 0.3 && traps.iter().count() < 5 {
        let x = (random::<f32>() * ARENA_WIDTH as f32) as i32;
        let y = (random::<f32>() * ARENA_HEIGHT as f32) as i32;
        let duration = 5.0 + random::<f32>() * 5.0; // 5 to 10 seconds

        let shapes = vec![
            vec![(0, 0)],
            vec![(0, 0), (1, 0), (2, 0)],         // 3x1 horizontal
            vec![(0, 0), (0, 1), (0, 2)],         // 1x3 vertical
            vec![(0, 0), (1, 0), (0, 1), (1, 1)], // 2x2 square
            vec![(0, 0), (1, 0), (2, 0), (1, 1), (1, -1)], // cross
        ];
        let shape = &shapes[random::<usize>() % shapes.len()];

        let trap_id = commands
            .spawn(Trap {
                timer: Timer::from_seconds(duration, TimerMode::Once),
            })
            .id();

        for &(dx, dy) in shape {
            let px = x + dx;
            let py = y + dy;
            if px >= 0 && py >= 0 && px < ARENA_WIDTH as i32 && py < ARENA_HEIGHT as i32 {
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            custom_size: Some(Vec2::ONE),
                            ..default()
                        },
                        texture: assets.bomb.clone(),
                        transform: Transform::from_xyz(0.0, 0.0, 1.2),
                        ..default()
                    },
                    Position { x: px, y: py },
                    Size::square(0.96),
                    Pulse {
                        amplitude: 0.05,
                        speed: 7.0,
                        phase: random::<f32>() * TAU,
                    },
                    TrapTile(trap_id),
                ));
            }
        }

        commands.spawn((
            Text2dBundle {
                text: Text::from_section(
                    duration.ceil().to_string(),
                    TextStyle {
                        font_size: 24.0,
                        color: col(COL_BG),
                        ..default()
                    },
                ),
                transform: Transform::from_xyz(0.0, 0.0, 2.5),
                ..default()
            },
            Position { x, y },
            TrapText(trap_id),
        ));
    }
}

pub fn update_traps(
    mut commands: Commands,
    time: Res<Time>,
    mut traps: Query<(Entity, &mut Trap)>,
    mut trap_texts: Query<(Entity, &mut Text, &TrapText)>,
    trap_tiles: Query<(Entity, &TrapTile)>,
) {
    for (entity, mut trap) in traps.iter_mut() {
        trap.timer.tick(time.delta());
        if trap.timer.finished() {
            commands.entity(entity).despawn();
            for (tile_ent, tile) in trap_tiles.iter() {
                if tile.0 == entity {
                    commands.entity(tile_ent).despawn();
                }
            }
        }
    }

    for (text_entity, mut text, trap_text) in trap_texts.iter_mut() {
        if let Ok((_, trap)) = traps.get(trap_text.0) {
            let remaining = trap.timer.remaining_secs().ceil() as i32;
            text.sections[0].value = remaining.to_string();
        } else {
            commands.entity(text_entity).despawn();
        }
    }
}
