use crate::components::*;
use crate::constants::*;
use crate::pixel_art::PixelAssets;
use bevy::prelude::*;
use std::collections::HashSet;
use std::f32::consts::{FRAC_PI_2, PI};

/// Z layers, kept together so the stacking order is easy to reason about.
const Z_BODY: f32 = 2.0;
const Z_HEAD: f32 = 2.1;

pub fn spawn_player(
    commands: &mut Commands,
    player: Player,
    segments: &mut Vec<Entity>,
    assets: &PixelAssets,
) {
    let x = match player {
        Player::One => (ARENA_WIDTH * 3 / 4) as i32,
        Player::Two => (ARENA_WIDTH / 4) as i32,
    };

    *segments = vec![
        commands
            .spawn((
                SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2::ONE),
                        ..default()
                    },
                    texture: assets.head(player, false),
                    transform: Transform::from_xyz(0.0, 0.0, Z_HEAD),
                    ..default()
                },
                SnakeHead {
                    direction: SnakeDirection::Up,
                },
                SnakeSegment,
                player,
                Position {
                    x,
                    y: (ARENA_HEIGHT / 2) as i32,
                },
                Size::square(1.04),
            ))
            .id(),
        spawn_segment(
            commands,
            Position {
                x,
                y: (ARENA_HEIGHT / 2 - 1) as i32,
            },
            player,
            assets,
        ),
    ];
}

pub fn spawn_snakes(
    mut commands: Commands,
    mut segments: ResMut<SnakeSegments>,
    assets: Res<PixelAssets>,
) {
    let mut p1_segs = Vec::new();
    spawn_player(&mut commands, Player::One, &mut p1_segs, &assets);
    segments.player1 = p1_segs;

    let mut p2_segs = Vec::new();
    spawn_player(&mut commands, Player::Two, &mut p2_segs, &assets);
    segments.player2 = p2_segs;
}

pub fn spawn_segment(
    commands: &mut Commands,
    position: Position,
    player: Player,
    assets: &PixelAssets,
) -> Entity {
    commands
        .spawn((
            SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::ONE),
                    ..default()
                },
                texture: assets.body(player),
                transform: Transform::from_xyz(0.0, 0.0, Z_BODY),
                ..default()
            },
            SnakeSegment,
            player,
            position,
            Size::square(1.0),
        ))
        .id()
}

/// Rotation for a sprite drawn facing up.
fn facing_angle(direction: SnakeDirection) -> f32 {
    match direction {
        SnakeDirection::Up => 0.0,
        SnakeDirection::Right => -FRAC_PI_2,
        SnakeDirection::Down => PI,
        SnakeDirection::Left => FRAC_PI_2,
    }
}

/// Rotation that points the tail's neck at `neighbor`. Steps of more than one
/// tile mean the snake wrapped around the arena, so the sign is flipped back.
fn neck_angle(tail: Position, neighbor: Position) -> f32 {
    let unwrap = |delta: i32| match delta {
        d if d > 1 => -1,
        d if d < -1 => 1,
        d => d,
    };
    let dx = unwrap(neighbor.x - tail.x);
    let dy = unwrap(neighbor.y - tail.y);

    if dy > 0 {
        0.0
    } else if dy < 0 {
        PI
    } else if dx > 0 {
        -FRAC_PI_2
    } else if dx < 0 {
        FRAC_PI_2
    } else {
        0.0
    }
}

/// Keeps head rotation, tail shape and tail rotation in sync with the snake's
/// current layout. Head *texture* is owned by [`animate_blink`].
pub fn update_snake_visuals(
    assets: Res<PixelAssets>,
    segments_res: Res<SnakeSegments>,
    heads: Query<&SnakeHead>,
    mut q: Query<(&Position, &mut Transform, &mut Handle<Image>), With<SnakeSegment>>,
) {
    for player in [Player::One, Player::Two] {
        let segments = match player {
            Player::One => &segments_res.player1,
            Player::Two => &segments_res.player2,
        };
        if segments.is_empty() {
            continue;
        }

        let positions: Vec<Position> = segments
            .iter()
            .filter_map(|entity| q.get(*entity).ok().map(|(pos, _, _)| *pos))
            .collect();
        if positions.len() != segments.len() {
            continue;
        }

        if let Ok(head) = heads.get(segments[0]) {
            if let Ok((_, mut transform, _)) = q.get_mut(segments[0]) {
                transform.rotation = Quat::from_rotation_z(facing_angle(head.direction));
            }
        }

        let last = segments.len() - 1;
        for (i, entity) in segments.iter().enumerate().skip(1) {
            let Ok((_, mut transform, mut texture)) = q.get_mut(*entity) else {
                continue;
            };

            let (wanted, rotation) = if i == last {
                (
                    assets.tail(player),
                    Quat::from_rotation_z(neck_angle(positions[i], positions[i - 1])),
                )
            } else {
                (assets.body(player), Quat::IDENTITY)
            };

            transform.rotation = rotation;
            if *texture != wanted {
                *texture = wanted;
            }
        }
    }
}

/// Occasional eye blink, offset per player so they never blink in lockstep.
pub fn animate_blink(
    time: Res<Time>,
    assets: Res<PixelAssets>,
    mut q: Query<(&Player, &mut Handle<Image>), With<SnakeHead>>,
) {
    const BLINK_PERIOD: f32 = 3.4;
    const BLINK_LENGTH: f32 = 0.14;

    for (player, mut texture) in q.iter_mut() {
        let offset = match player {
            Player::One => 0.0,
            Player::Two => BLINK_PERIOD / 2.0,
        };
        let blinking = (time.elapsed_seconds() + offset) % BLINK_PERIOD < BLINK_LENGTH;

        let wanted = assets.head(*player, blinking);
        if *texture != wanted {
            *texture = wanted;
        }
    }
}

pub fn snake_movement_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut heads: Query<(&mut SnakeHead, &Player)>,
) {
    for (mut head, player) in heads.iter_mut() {
        let dir: SnakeDirection = match player {
            Player::One => {
                if keyboard_input.pressed(KeyCode::ArrowLeft) {
                    SnakeDirection::Left
                } else if keyboard_input.pressed(KeyCode::ArrowDown) {
                    SnakeDirection::Down
                } else if keyboard_input.pressed(KeyCode::ArrowUp) {
                    SnakeDirection::Up
                } else if keyboard_input.pressed(KeyCode::ArrowRight) {
                    SnakeDirection::Right
                } else {
                    head.direction
                }
            }
            Player::Two => {
                if keyboard_input.pressed(KeyCode::KeyA) {
                    SnakeDirection::Left
                } else if keyboard_input.pressed(KeyCode::KeyS) {
                    SnakeDirection::Down
                } else if keyboard_input.pressed(KeyCode::KeyW) {
                    SnakeDirection::Up
                } else if keyboard_input.pressed(KeyCode::KeyD) {
                    SnakeDirection::Right
                } else {
                    head.direction
                }
            }
        };
        if dir != head.direction.opposite() {
            head.direction = dir;
        }
    }
}

pub fn snake_movement(
    mut positions: Query<&mut Position, With<SnakeSegment>>,
    heads: Query<(Entity, &SnakeHead, &Player)>,
    segments_res: Res<SnakeSegments>,
    traps: Query<&Position, (With<TrapTile>, Without<SnakeSegment>)>,
    mut death_writer: EventWriter<PlayerDiedEvent>,
    p_states: Res<PlayerStates>,
) {
    let mut new_head_positions = Vec::new();
    let mut dead_players = HashSet::new();

    for (head_entity, head, player) in heads.iter() {
        let is_respawning = match player {
            Player::One => p_states.p1_respawn_timer.is_some(),
            Player::Two => p_states.p2_respawn_timer.is_some(),
        };
        if is_respawning {
            continue;
        }

        let player_segments = match player {
            Player::One => &segments_res.player1,
            Player::Two => &segments_res.player2,
        };

        if player_segments.is_empty() {
            continue;
        }

        let mut last_position = if let Ok(p) = positions.get(head_entity) {
            *p
        } else {
            continue;
        };

        let mut new_head_pos = last_position;
        match &head.direction {
            SnakeDirection::Left => new_head_pos.x -= 1,
            SnakeDirection::Right => new_head_pos.x += 1,
            SnakeDirection::Up => new_head_pos.y += 1,
            SnakeDirection::Down => new_head_pos.y -= 1,
        }

        if new_head_pos.x < 0 {
            new_head_pos.x = (ARENA_WIDTH - 1) as i32;
        } else if new_head_pos.x >= ARENA_WIDTH as i32 {
            new_head_pos.x = 0;
        }

        if new_head_pos.y < 0 {
            new_head_pos.y = (ARENA_HEIGHT - 1) as i32;
        } else if new_head_pos.y >= ARENA_HEIGHT as i32 {
            new_head_pos.y = 0;
        }

        new_head_positions.push((head_entity, new_head_pos, *player));

        let mut current_pos = last_position;
        for (i, entity) in player_segments.iter().enumerate() {
            if i == 0 {
                continue;
            }
            if let Ok(mut pos) = positions.get_mut(*entity) {
                last_position = *pos;
                *pos = current_pos;
                current_pos = last_position;
            }
        }
    }

    for (head_entity, new_pos, _) in &new_head_positions {
        if let Ok(mut p) = positions.get_mut(*head_entity) {
            *p = *new_pos;
        }
    }

    for (head_entity, new_pos, player) in &new_head_positions {
        for ent in segments_res
            .player1
            .iter()
            .chain(segments_res.player2.iter())
        {
            if *ent != *head_entity {
                if let Ok(pos) = positions.get(*ent) {
                    if *pos == *new_pos {
                        dead_players.insert(*player);
                    }
                }
            }
        }
        for trap_pos in traps.iter() {
            if *trap_pos == *new_pos {
                dead_players.insert(*player);
            }
        }
    }

    for player in dead_players {
        death_writer.send(PlayerDiedEvent(player));
    }
}

pub fn snake_eating(
    mut commands: Commands,
    mut growth_writer: EventWriter<GrowthEvent>,
    mut scores: ResMut<Scores>,
    food_positions: Query<(Entity, &Position), With<Food>>,
    head_positions: Query<(&Position, &Player), With<SnakeHead>>,
    p_states: Res<PlayerStates>,
) {
    for (head_pos, player) in head_positions.iter() {
        let is_respawning = match player {
            Player::One => p_states.p1_respawn_timer.is_some(),
            Player::Two => p_states.p2_respawn_timer.is_some(),
        };
        if is_respawning {
            continue;
        }

        for (ent, food_pos) in food_positions.iter() {
            if food_pos == head_pos {
                commands.entity(ent).despawn();
                growth_writer.send(GrowthEvent(*player));
                match player {
                    Player::One => scores.player1 += 1,
                    Player::Two => scores.player2 += 1,
                }
            }
        }
    }
}

pub fn snake_growth(
    mut commands: Commands,
    mut segments: ResMut<SnakeSegments>,
    mut growth_reader: EventReader<GrowthEvent>,
    q_segments: Query<&Position>,
    assets: Res<PixelAssets>,
) {
    for ev in growth_reader.read() {
        let player_segments = match ev.0 {
            Player::One => &mut segments.player1,
            Player::Two => &mut segments.player2,
        };
        if let Some(last_segment) = player_segments.last() {
            if let Ok(pos) = q_segments.get(*last_segment) {
                player_segments.push(spawn_segment(&mut commands, *pos, ev.0, &assets));
            }
        }
    }
}

pub fn handle_player_death(
    mut commands: Commands,
    mut reader: EventReader<PlayerDiedEvent>,
    mut segments_res: ResMut<SnakeSegments>,
    mut scores: ResMut<Scores>,
    mut positions: Query<&mut Position, With<SnakeSegment>>,
    mut heads: Query<&mut SnakeHead>,
    mut p_states: ResMut<PlayerStates>,
    mut p_hp: ResMut<PlayerHP>,
) {
    for event in reader.read() {
        let player = event.0;

        let is_dead = match player {
            Player::One => {
                p_hp.player1 = p_hp.player1.saturating_sub(1);
                p_hp.player1 == 0
            }
            Player::Two => {
                p_hp.player2 = p_hp.player2.saturating_sub(1);
                p_hp.player2 == 0
            }
        };

        let player_segments = match player {
            Player::One => &mut segments_res.player1,
            Player::Two => &mut segments_res.player2,
        };

        if is_dead {
            for &ent in player_segments.iter() {
                commands.entity(ent).despawn();
            }
            player_segments.clear();
            continue;
        }

        match player {
            Player::One => {
                p_states.p1_respawn_timer = Some(Timer::from_seconds(3.0, TimerMode::Once))
            }
            Player::Two => {
                p_states.p2_respawn_timer = Some(Timer::from_seconds(3.0, TimerMode::Once))
            }
        }

        match player {
            Player::One => scores.player1 /= 2,
            Player::Two => scores.player2 /= 2,
        }

        let body_count = player_segments.len().saturating_sub(1);
        let keep_body = body_count / 2;
        let keep_total = 1 + keep_body;

        while player_segments.len() > keep_total {
            if let Some(ent) = player_segments.pop() {
                commands.entity(ent).despawn();
            }
        }

        let start_x = match player {
            Player::One => (ARENA_WIDTH * 3 / 4) as i32,
            Player::Two => (ARENA_WIDTH / 4) as i32,
        };
        let start_y = (ARENA_HEIGHT / 2) as i32;

        if let Some(&head_ent) = player_segments.first() {
            if let Ok(mut head) = heads.get_mut(head_ent) {
                head.direction = SnakeDirection::Up;
            }
        }

        for (i, &ent) in player_segments.iter().enumerate() {
            if let Ok(mut pos) = positions.get_mut(ent) {
                pos.x = start_x;
                pos.y = start_y - i as i32;
            }
        }

        commands.spawn((
            Text2dBundle {
                text: Text::from_section(
                    "3",
                    TextStyle {
                        font_size: 46.0,
                        color: col(COL_INK),
                        ..default()
                    },
                ),
                transform: Transform::from_xyz(0.0, 0.0, 3.0),
                ..default()
            },
            Position {
                x: start_x,
                y: start_y,
            },
            RespawnText(player),
        ));
    }
}

pub fn update_respawn_timers(
    mut commands: Commands,
    time: Res<Time>,
    mut states: ResMut<PlayerStates>,
    mut q_sprites: Query<(&Player, &mut Sprite), With<SnakeSegment>>,
    mut q_texts: Query<(Entity, &mut Text, &RespawnText)>,
) {
    let blink = (time.elapsed_seconds() * 10.0) as u32 % 2 == 0;

    if let Some(timer) = &mut states.p1_respawn_timer {
        timer.tick(time.delta());
        if timer.finished() {
            states.p1_respawn_timer = None;
        }
    }
    if let Some(timer) = &mut states.p2_respawn_timer {
        timer.tick(time.delta());
        if timer.finished() {
            states.p2_respawn_timer = None;
        }
    }

    for (player, mut sprite) in q_sprites.iter_mut() {
        let is_respawning = match player {
            Player::One => states.p1_respawn_timer.is_some(),
            Player::Two => states.p2_respawn_timer.is_some(),
        };
        if is_respawning {
            sprite.color.set_alpha(if blink { 0.25 } else { 1.0 });
        } else {
            sprite.color.set_alpha(1.0);
        }
    }

    for (ent, mut text, respawn_text) in q_texts.iter_mut() {
        let timer_opt = match respawn_text.0 {
            Player::One => &states.p1_respawn_timer,
            Player::Two => &states.p2_respawn_timer,
        };
        if let Some(timer) = timer_opt {
            text.sections[0].value = timer.remaining_secs().ceil().to_string();
        } else {
            commands.entity(ent).despawn();
        }
    }
}

pub fn restart_game(
    mut commands: Commands,
    mut reader: EventReader<RestartGameEvent>,
    mut segments_res: ResMut<SnakeSegments>,
    mut scores: ResMut<Scores>,
    mut p_states: ResMut<PlayerStates>,
    mut p_hp: ResMut<PlayerHP>,
    settings: Res<GameSettings>,
    assets: Res<PixelAssets>,
    q_segments: Query<Entity, With<SnakeSegment>>,
    q_food: Query<Entity, With<Food>>,
    q_traps: Query<Entity, With<Trap>>,
    q_trap_texts: Query<Entity, With<TrapText>>,
    q_trap_tiles: Query<Entity, With<TrapTile>>,
    q_respawn_texts: Query<Entity, With<RespawnText>>,
) {
    for _ in reader.read() {
        for e in q_segments.iter() {
            commands.entity(e).despawn();
        }
        for e in q_food.iter() {
            commands.entity(e).despawn();
        }
        for e in q_traps.iter() {
            commands.entity(e).despawn();
        }
        for e in q_trap_texts.iter() {
            commands.entity(e).despawn();
        }
        for e in q_trap_tiles.iter() {
            commands.entity(e).despawn();
        }
        for e in q_respawn_texts.iter() {
            commands.entity(e).despawn();
        }

        scores.player1 = 0;
        scores.player2 = 0;
        p_hp.player1 = settings.initial_hp;
        p_hp.player2 = settings.initial_hp;

        let mut p1_segs = Vec::new();
        spawn_player(&mut commands, Player::One, &mut p1_segs, &assets);
        segments_res.player1 = p1_segs;

        let mut p2_segs = Vec::new();
        spawn_player(&mut commands, Player::Two, &mut p2_segs, &assets);
        segments_res.player2 = p2_segs;

        p_states.p1_respawn_timer = None;
        p_states.p2_respawn_timer = None;
    }
}
