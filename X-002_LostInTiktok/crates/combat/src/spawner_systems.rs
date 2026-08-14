use crate::components::*;
use bevy::prelude::*;
use rand::Rng;
use soulforge_core::constants::*;
use soulforge_core::resources::RunTimer;
use soulforge_data::loader::GameDatabase;

#[derive(Resource)]
pub struct WaveSpawnerTimer {
    pub timer: Timer,
}

impl Default for WaveSpawnerTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
        }
    }
}

/// 怪物波次生成系统（读取波次配置，环形生成怪物群）
pub fn wave_spawner_system(
    time: Res<Time>,
    run_timer: Res<RunTimer>,
    mut spawner_timer: ResMut<WaveSpawnerTimer>,
    db: Res<GameDatabase>,
    player_query: Query<&Transform, With<Player>>,
    enemy_query: Query<&Enemy>,
    mut commands: Commands,
) {
    let Ok(player_tf) = player_query.get_single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();
    let current_time = run_timer.total_seconds;

    // 寻找当前生效的波次
    let active_wave = db.waves.waves.iter().find(|w| {
        current_time >= w.time_start && current_time < w.time_end
    });

    let Some(wave) = active_wave else {
        return;
    };

    spawner_timer.timer.set_duration(std::time::Duration::from_secs_f32(wave.spawn_interval));
    spawner_timer.timer.tick(time.delta());

    if !spawner_timer.timer.just_finished() {
        return;
    }

    let current_enemy_count = enemy_query.iter().count();
    if current_enemy_count >= wave.max_concurrent {
        return;
    }

    let mut rng = rand::thread_rng();
    let total_weight: u32 = wave.enemies.iter().map(|e| e.weight).sum();
    if total_weight == 0 {
        return;
    }

    let spawn_count = wave.spawn_batch_size.min(wave.max_concurrent.saturating_sub(current_enemy_count));

    for _ in 0..spawn_count {
        // 根据权重随机抽取怪物类型
        let mut roll = rng.gen_range(0..total_weight);
        let mut chosen_id = &wave.enemies[0].id;
        for enemy_entry in &wave.enemies {
            if roll < enemy_entry.weight {
                chosen_id = &enemy_entry.id;
                break;
            }
            roll -= enemy_entry.weight;
        }

        let Some(enemy_cfg) = db.enemies.enemies.get(chosen_id) else {
            continue;
        };

        // 环形生成坐标 (在玩家视野边缘 240px ~ 320px 环内)
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let distance = rng.gen_range(240.0..320.0);
        let spawn_pos = player_pos + Vec2::new(angle.cos() * distance, angle.sin() * distance);

        let color = if enemy_cfg.is_explosive {
            Color::srgba(1.0, 0.25, 0.25, 0.9)
        } else if enemy_cfg.is_boss {
            Color::srgba(0.9, 0.2, 0.1, 1.0)
        } else if enemy_cfg.is_ranged {
            Color::srgba(0.6, 0.3, 0.8, 0.9)
        } else {
            Color::srgba(0.85, 0.88, 0.92, 0.9)
        };

        commands.spawn((
            SpriteBundle {
                transform: Transform::from_xyz(spawn_pos.x, spawn_pos.y, Z_ENEMIES),
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::splat(enemy_cfg.size)),
                    ..default()
                },
                ..default()
            },
            CombatStats {
                max_hp: enemy_cfg.max_hp,
                current_hp: enemy_cfg.max_hp,
                base_damage: enemy_cfg.damage,
                move_speed: enemy_cfg.move_speed,
                armor: enemy_cfg.armor,
            },
            Enemy {
                id: enemy_cfg.id.clone(),
                name: enemy_cfg.name.clone(),
                max_hp: enemy_cfg.max_hp,
                damage: enemy_cfg.damage,
                move_speed: enemy_cfg.move_speed,
                armor: enemy_cfg.armor,
                exp_reward: enemy_cfg.exp_reward,
                gold_reward: enemy_cfg.gold_reward,
                is_explosive: enemy_cfg.is_explosive,
                is_ranged: enemy_cfg.is_ranged,
                is_boss: enemy_cfg.is_boss,
                attack_cooldown: Timer::from_seconds(enemy_cfg.attack_cooldown, TimerMode::Repeating),
                size: enemy_cfg.size,
                exploding: false,
                explosion_timer: Timer::from_seconds(0.6, TimerMode::Once),
            },
        ));
    }
}
