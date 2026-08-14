use crate::components::*;
use bevy::prelude::*;
use soulforge_core::constants::*;
use soulforge_core::events::{DamageEvent, LootDropEvent, SpawnDamageTextEvent};
use soulforge_core::spatial_hash::SpatialHash2D;
use soulforge_data::loader::GameDatabase;

/// 投射物移动、空间碰撞判定与生命周期系统
pub fn projectile_update_system(
    time: Res<Time>,
    spatial_hash: Res<SpatialHash2D>,
    player_query: Query<(Entity, &Transform, &Player), Without<Enemy>>,
    mut proj_query: Query<(Entity, &mut Transform, &mut Projectile), Without<Player>>,
    mut damage_events: EventWriter<DamageEvent>,
    mut commands: Commands,
) {
    let dt = time.delta_seconds();
    let player_data = player_query.get_single();

    for (proj_entity, mut proj_tf, mut proj) in proj_query.iter_mut() {
        // 移动
        proj_tf.translation.x += proj.velocity.x * dt;
        proj_tf.translation.y += proj.velocity.y * dt;

        proj.lifetime.tick(time.delta());
        if proj.lifetime.finished() {
            commands.entity(proj_entity).despawn_recursive();
            continue;
        }

        let proj_pos = proj_tf.translation.truncate();

        if proj.is_player {
            // 玩家投射物：通过空间哈希查询命中怪物
            let candidates = spatial_hash.query_radius(proj_pos, proj.radius + 12.0);
            for (enemy_entity, enemy_pos) in candidates {
                if proj.hit_entities.contains(&enemy_entity) {
                    continue;
                }

                if proj_pos.distance(enemy_pos) <= proj.radius + 8.0 {
                    proj.hit_entities.insert(enemy_entity);

                    damage_events.send(DamageEvent {
                        source: proj_entity,
                        target: enemy_entity,
                        base_damage: proj.damage,
                        is_bleed: false,
                        is_crit: false,
                    });

                    if proj.pierce_remaining > 0 {
                        proj.pierce_remaining -= 1;
                    }
                    if proj.pierce_remaining == 0 {
                        commands.entity(proj_entity).despawn_recursive();
                        break;
                    }
                }
            }
        } else {
            // 敌方投射物：判定是否命中玩家
            if let Ok((player_entity, player_tf, player)) = player_data {
                let player_pos = player_tf.translation.truncate();
                if proj_pos.distance(player_pos) <= proj.radius + 10.0 && !player.is_dashing {
                    damage_events.send(DamageEvent {
                        source: proj_entity,
                        target: player_entity,
                        base_damage: proj.damage,
                        is_bleed: false,
                        is_crit: false,
                    });
                    commands.entity(proj_entity).despawn_recursive();
                }
            }
        }
    }
}

/// 近战挥砍判定体命中与淡出系统
pub fn melee_slash_update_system(
    time: Res<Time>,
    spatial_hash: Res<SpatialHash2D>,
    mut slash_query: Query<(Entity, &mut MeleeSlash, &Transform, &mut Sprite)>,
    mut damage_events: EventWriter<DamageEvent>,
    mut commands: Commands,
) {
    for (entity, mut slash, tf, mut sprite) in slash_query.iter_mut() {
        slash.lifetime.tick(time.delta());
        let slash_pos = tf.translation.truncate();

        // 渐隐淡出
        let progress = slash.lifetime.fraction();
        let alpha = (1.0 - progress).max(0.0);
        sprite.color = Color::srgba(1.0, 1.0, 1.0, alpha);

        if slash.lifetime.finished() {
            commands.entity(entity).despawn_recursive();
            continue;
        }

        // 使用空间哈希进行判定
        let candidates = spatial_hash.query_radius(slash_pos, slash.radius);
        for (enemy_entity, _) in candidates {
            if !slash.hit_entities.contains(&enemy_entity) {
                slash.hit_entities.insert(enemy_entity);

                damage_events.send(DamageEvent {
                    source: entity,
                    target: enemy_entity,
                    base_damage: slash.damage,
                    is_bleed: false,
                    is_crit: false,
                });
            }
        }
    }
}

/// 监听掉落物生成事件系统
pub fn handle_loot_drop_events_system(
    mut loot_events: EventReader<LootDropEvent>,
    db: Res<GameDatabase>,
    mut commands: Commands,
) {
    for event in loot_events.read() {
        let color = if let Some(cfg) = db.items.items.get(&event.item_id) {
            match cfg.item_type {
                soulforge_data::models::ItemType::Weapon => Color::srgb(0.3, 0.7, 1.0),
                soulforge_data::models::ItemType::Material => Color::srgb(0.9, 0.4, 0.2),
                soulforge_data::models::ItemType::Consumable => Color::srgb(0.2, 0.9, 0.3),
                soulforge_data::models::ItemType::Artifact => Color::srgb(0.9, 0.8, 0.1),
            }
        } else {
            Color::srgb(1.0, 1.0, 1.0)
        };

        commands.spawn((
            SpriteBundle {
                transform: Transform::from_xyz(event.world_position.x, event.world_position.y, Z_LOOT),
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::splat(10.0)),
                    ..default()
                },
                ..default()
            },
            LootItem {
                item_id: event.item_id.clone(),
                magnet_active: false,
                velocity: Vec2::ZERO,
                despawn_timer: Timer::from_seconds(60.0, TimerMode::Once),
            },
        ));
    }
}

/// 浮动伤害跳字生成与漂浮淡出系统
pub fn floating_damage_text_system(
    mut text_events: EventReader<SpawnDamageTextEvent>,
    time: Res<Time>,
    mut text_query: Query<(Entity, &mut Transform, &mut FloatingDamageText)>,
    mut commands: Commands,
) {
    for event in text_events.read() {
        let (color, font_size, text_content) = if event.is_heal {
            (Color::srgb(0.2, 0.9, 0.3), 13.0, format!("+{}", event.amount as u32))
        } else if event.is_crit {
            (Color::srgb(1.0, 0.9, 0.1), 16.0, format!("{}!", event.amount as u32))
        } else if event.is_bleed {
            (Color::srgb(0.9, 0.2, 0.4), 11.0, format!("~{}", event.amount as u32))
        } else {
            (Color::srgb(1.0, 1.0, 1.0), 12.0, format!("{}", event.amount as u32))
        };

        commands.spawn((
            Text2dBundle {
                text: Text::from_section(
                    text_content,
                    TextStyle {
                        font_size,
                        color,
                        ..default()
                    },
                ),
                transform: Transform::from_xyz(event.position.x, event.position.y, Z_DAMAGE_TEXT),
                ..default()
            },
            FloatingDamageText {
                timer: Timer::from_seconds(0.7, TimerMode::Once),
                velocity: Vec2::new(0.0, 24.0),
                initial_position: event.position,
            },
        ));
    }

    // 漂浮与淡出
    let dt = time.delta_seconds();
    for (entity, mut tf, mut float_data) in text_query.iter_mut() {
        float_data.timer.tick(time.delta());
        tf.translation.y += float_data.velocity.y * dt;

        if float_data.timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
