use crate::components::*;
use bevy::prelude::*;
use soulforge_core::constants::*;
use soulforge_core::events::DamageEvent;
use soulforge_core::spatial_hash::SpatialHash2D;

/// 每帧重建怪物空间哈希表系统
pub fn update_spatial_hash_system(
    mut spatial_hash: ResMut<SpatialHash2D>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
) {
    spatial_hash.clear();
    for (entity, transform) in enemy_query.iter() {
        spatial_hash.insert(entity, transform.translation.truncate());
    }
}

/// 怪物群体 AI 与批处理位移系统（融合向玩家靠拢与 Boids 空间哈希排斥力）
pub fn enemy_swarm_ai_system(
    time: Res<Time>,
    spatial_hash: Res<SpatialHash2D>,
    player_query: Query<(Entity, &Transform, &Player), Without<Enemy>>,
    mut enemy_query: Query<(
        Entity,
        &mut Transform,
        &mut Enemy,
        &CombatStats,
        Option<&mut Sprite>,
    )>,
    mut damage_events: EventWriter<DamageEvent>,
    mut commands: Commands,
) {
    let Ok((player_entity, player_tf, player)) = player_query.get_single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();
    let dt = time.delta_seconds();

    for (enemy_entity, mut enemy_tf, mut enemy, stats, maybe_sprite) in enemy_query.iter_mut() {
        let current_pos = enemy_tf.translation.truncate();
        let to_player = player_pos - current_pos;
        let dist = to_player.length();

        // 1. 特殊怪物行为：自爆怪
        if enemy.is_explosive {
            if dist < 32.0 && !enemy.exploding {
                enemy.exploding = true;
                enemy.explosion_timer = Timer::from_seconds(0.6, TimerMode::Once);
            }

            if enemy.exploding {
                enemy.explosion_timer.tick(time.delta());
                // 闪烁特效
                if let Some(mut sprite) = maybe_sprite {
                    let flash = (enemy.explosion_timer.elapsed_secs() * 20.0).sin().abs();
                    sprite.color = Color::srgba(1.0, 0.2 * flash, 0.2 * flash, 1.0);
                }

                if enemy.explosion_timer.just_finished() {
                    // 自爆范围伤害
                    let blast_radius = 48.0;
                    if player_pos.distance(current_pos) <= blast_radius && !player.is_dashing {
                        damage_events.send(DamageEvent {
                            source: enemy_entity,
                            target: player_entity,
                            base_damage: enemy.damage * 1.5,
                            is_bleed: false,
                            is_crit: false,
                        });
                    }
                    commands.entity(enemy_entity).despawn_recursive();
                    continue;
                }
            }
        }

        // 2. 远程怪开火逻辑
        if enemy.is_ranged && !enemy.exploding {
            enemy.attack_cooldown.tick(time.delta());
            if dist < 180.0 && dist > 70.0 && enemy.attack_cooldown.just_finished() {
                // 远程开火：发射暗影弹幕
                let shoot_dir = to_player.normalize_or_zero();
                commands.spawn((
                    SpriteBundle {
                        transform: Transform::from_xyz(current_pos.x, current_pos.y, Z_PROJECTILES),
                        sprite: Sprite {
                            color: Color::srgb(0.8, 0.2, 0.9),
                            custom_size: Some(Vec2::splat(8.0)),
                            ..default()
                        },
                        ..default()
                    },
                    Projectile {
                        velocity: shoot_dir * 110.0,
                        damage: enemy.damage,
                        pierce_remaining: 1,
                        hit_entities: Default::default(),
                        lifetime: Timer::from_seconds(4.0, TimerMode::Once),
                        is_player: false,
                        radius: 6.0,
                        affixes: Vec::new(),
                    },
                ));
            }
        }

        // 3. 计算群体排斥力与追踪速度
        let dir_to_player = if dist > 0.001 {
            to_player / dist
        } else {
            Vec2::ZERO
        };

        // 空间哈希计算邻居排斥力 (Boids Separation)
        let separation = spatial_hash.compute_separation(
            enemy_entity,
            current_pos,
            enemy.size.max(16.0),
            6,
        );

        let final_dir = (dir_to_player * 0.65 + separation * 0.35).normalize_or_zero();
        let speed = if enemy.exploding {
            stats.move_speed * 1.25
        } else {
            stats.move_speed
        };

        let new_pos = current_pos + final_dir * speed * dt;
        enemy_tf.translation.x = new_pos.x;
        enemy_tf.translation.y = new_pos.y;

        // 4. 近战怪物触碰伤害
        enemy.attack_cooldown.tick(time.delta());
        let contact_dist = (enemy.size * 0.5) + 12.0;
        if dist <= contact_dist && enemy.attack_cooldown.just_finished() && !player.is_dashing {
            damage_events.send(DamageEvent {
                source: enemy_entity,
                target: player_entity,
                base_damage: enemy.damage,
                is_bleed: false,
                is_crit: false,
            });
        }
    }
}
