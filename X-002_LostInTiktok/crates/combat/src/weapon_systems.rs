use crate::components::*;
use bevy::prelude::*;
use soulforge_core::constants::*;
use soulforge_core::spatial_hash::SpatialHash2D;

/// 武器自动寻敌攻击系统
pub fn weapon_auto_attack_system(
    time: Res<Time>,
    spatial_hash: Res<SpatialHash2D>,
    player_query: Query<(&Transform, &Player), (With<Player>, Without<Enemy>)>,
    mut weapon_query: Query<(&mut Weapon, Option<&Affixes>)>,
    mut commands: Commands,
) {
    let Ok((player_tf, _)) = player_query.get_single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();

    for (mut weapon, maybe_affixes) in weapon_query.iter_mut() {
        if !weapon.active {
            continue;
        }

        // 计算词条修饰对攻速和射程的加成
        let mut speed_bonus = 0.0;
        let mut range_bonus = 0.0;
        let mut pierce_bonus: u8 = 0;
        let mut affixes_vec = Vec::new();

        if let Some(affixes) = maybe_affixes {
            affixes_vec = affixes.mods.clone();
            for mod_item in &affixes.mods {
                match mod_item {
                    AffixModifier::AttackSpeed(v) => speed_bonus += v,
                    AffixModifier::RangeBoost(v) => range_bonus += v,
                    AffixModifier::Pierce(v) => pierce_bonus += v,
                    _ => {}
                }
            }
        }

        let effective_range = weapon.attack_range * (1.0 + range_bonus);
        let _ = speed_bonus; // 攻速预留

        weapon.cooldown_timer.tick(time.delta());

        if weapon.cooldown_timer.just_finished() {
            // 通过空间哈希寻找最近的敌人
            let nearby = spatial_hash.query_radius(player_pos, effective_range);
            let mut target_dir = Vec2::X; // 默认朝向

            if let Some((_, closest_pos)) = nearby
                .into_iter()
                .min_by(|a, b| {
                    let d1 = a.1.distance_squared(player_pos);
                    let d2 = b.1.distance_squared(player_pos);
                    d1.partial_cmp(&d2).unwrap_or(std::cmp::Ordering::Equal)
                })
            {
                let diff = closest_pos - player_pos;
                if diff.length_squared() > 0.001 {
                    target_dir = diff.normalize();
                }
            }

            // 根据武器类型生成判定实体或弹幕
            match weapon.weapon_type {
                WeaponType::MeleeSlash | WeaponType::SwiftDagger => {
                    let slash_size = if weapon.weapon_type == WeaponType::SwiftDagger { 40.0 } else { 65.0 };
                    let slash_duration = if weapon.weapon_type == WeaponType::SwiftDagger { 0.12 } else { 0.2 };
                    let angle = target_dir.y.atan2(target_dir.x);

                    // 实例化近战挥砍判定体
                    commands.spawn((
                        SpriteBundle {
                            transform: Transform::from_xyz(
                                player_pos.x + target_dir.x * 24.0,
                                player_pos.y + target_dir.y * 24.0,
                                Z_SLASH_VFX,
                            ).with_rotation(Quat::from_rotation_z(angle)),
                            sprite: Sprite {
                                color: Color::srgba(0.9, 0.9, 1.0, 0.85),
                                custom_size: Some(Vec2::new(slash_size, slash_size * 0.5)),
                                ..default()
                            },
                            ..default()
                        },
                        MeleeSlash {
                            damage: if weapon.weapon_type == WeaponType::SwiftDagger { 8.0 } else { 15.0 },
                            lifetime: Timer::from_seconds(slash_duration, TimerMode::Once),
                            hit_entities: Default::default(),
                            radius: slash_size * 0.6,
                            arc_angle: std::f32::consts::FRAC_PI_2,
                            center_direction: target_dir,
                            affixes: affixes_vec,
                        },
                    ));
                }
                WeaponType::Crossbow => {
                    let count = weapon.projectile_count.max(1);
                    let spread_angle = 0.15;

                    for i in 0..count {
                        let offset = (i as f32 - (count - 1) as f32 / 2.0) * spread_angle;
                        let rot_dir = Vec2::new(
                            target_dir.x * offset.cos() - target_dir.y * offset.sin(),
                            target_dir.x * offset.sin() + target_dir.y * offset.cos(),
                        ).normalize();

                        commands.spawn((
                            SpriteBundle {
                                transform: Transform::from_xyz(player_pos.x, player_pos.y, Z_PROJECTILES)
                                    .with_rotation(Quat::from_rotation_z(rot_dir.y.atan2(rot_dir.x))),
                                sprite: Sprite {
                                    color: Color::srgb(0.7, 0.4, 1.0),
                                    custom_size: Some(Vec2::new(14.0, 4.0)),
                                    ..default()
                                },
                                ..default()
                            },
                            Projectile {
                                velocity: rot_dir * 280.0,
                                damage: 22.0,
                                pierce_remaining: 2 + pierce_bonus,
                                hit_entities: Default::default(),
                                lifetime: Timer::from_seconds(2.5, TimerMode::Once),
                                is_player: true,
                                radius: 8.0,
                                affixes: affixes_vec.clone(),
                            },
                        ));
                    }
                }
                WeaponType::ArcaneOrb => {
                    let count = weapon.projectile_count.max(1);
                    for i in 0..count {
                        let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
                        let dir = Vec2::new(angle.cos(), angle.sin());

                        commands.spawn((
                            SpriteBundle {
                                transform: Transform::from_xyz(player_pos.x, player_pos.y, Z_PROJECTILES),
                                sprite: Sprite {
                                    color: Color::srgb(0.2, 0.8, 1.0),
                                    custom_size: Some(Vec2::splat(10.0)),
                                    ..default()
                                },
                                ..default()
                            },
                            Projectile {
                                velocity: dir * 160.0,
                                damage: 16.0,
                                pierce_remaining: 1 + pierce_bonus,
                                hit_entities: Default::default(),
                                lifetime: Timer::from_seconds(3.0, TimerMode::Once),
                                is_player: true,
                                radius: 8.0,
                                affixes: affixes_vec.clone(),
                            },
                        ));
                    }
                }
            }
        }
    }
}
