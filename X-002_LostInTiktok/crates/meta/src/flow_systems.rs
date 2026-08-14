use crate::save_data::{PersistentSaveData, StoredItemData};
use bevy::prelude::*;
use rand::Rng;
use soulforge_combat::components::{CombatStats, Player};
use soulforge_core::constants::*;
use soulforge_core::events::ExtractionEvent;
use soulforge_core::resources::{ExtractionPoint, RunSessionContext, RunTimer};
use soulforge_core::states::AppState;
use soulforge_inventory::components::ItemComponent;
use soulforge_inventory::inventory::Inventory;
use soulforge_inventory::item_shape::ItemShape;

/// 局内计时与撤离点刷新监控系统
pub fn extraction_spawner_timer_system(
    time: Res<Time>,
    mut run_timer: ResMut<RunTimer>,
    mut session: ResMut<RunSessionContext>,
    player_query: Query<&Transform, With<Player>>,
    extraction_query: Query<&ExtractionPoint>,
    mut commands: Commands,
) {
    run_timer.total_seconds += time.delta_seconds();
    session.time_survived = run_timer.total_seconds;

    // 当达到撤离刷新时间点，且场上无活跃撤离点时生成
    if run_timer.total_seconds >= run_timer.next_extraction_spawn_time && extraction_query.is_empty() {
        if let Ok(player_tf) = player_query.get_single() {
            let mut rng = rand::thread_rng();
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let dist = rng.gen_range(280.0..360.0); // 玩家可视范围边缘
            let spawn_pos = player_tf.translation.truncate() + Vec2::new(angle.cos() * dist, angle.sin() * dist);

            commands.spawn((
                SpriteBundle {
                    transform: Transform::from_xyz(spawn_pos.x, spawn_pos.y, Z_EXTRACTION_PORTAL),
                    sprite: Sprite {
                        color: Color::srgba(0.2, 0.9, 0.6, 0.85),
                        custom_size: Some(Vec2::splat(40.0)),
                        ..default()
                    },
                    ..default()
                },
                ExtractionPoint {
                    active: true,
                    countdown_timer: Timer::from_seconds(50.0, TimerMode::Once),
                    radius: 44.0,
                    channel_timer: Timer::from_seconds(3.0, TimerMode::Once),
                },
            ));

            session.extraction_available = true;
            run_timer.next_extraction_spawn_time += 60.0;
            info!("Extraction mining cart spawned at position {:?}", spawn_pos);
        }
    }
}

/// 玩家在撤离圈内的引导与撤离触发系统
pub fn extraction_channeling_system(
    time: Res<Time>,
    mut extraction_query: Query<(Entity, &mut ExtractionPoint, &Transform, &mut Sprite)>,
    player_query: Query<(Entity, &Transform, &CombatStats), With<Player>>,
    inv_query: Query<&Inventory>,
    mut extraction_events: EventWriter<ExtractionEvent>,
    mut commands: Commands,
) {
    let Ok((player_entity, player_tf, player_stats)) = player_query.get_single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();

    // 玩家死亡检测
    if player_stats.current_hp <= 0.0 {
        extraction_events.send(ExtractionEvent::Death {
            kept_items: vec![player_entity],
        });
        return;
    }

    for (entity, mut extraction, tf, mut sprite) in extraction_query.iter_mut() {
        extraction.countdown_timer.tick(time.delta());
        let portal_pos = tf.translation.truncate();
        let dist = player_pos.distance(portal_pos);

        if dist <= extraction.radius {
            // 玩家在圈内，进行引导充能
            extraction.channel_timer.tick(time.delta());

            // 圈内光效加强
            let frac = extraction.channel_timer.fraction();
            sprite.color = Color::srgba(0.2 + 0.6 * frac, 1.0, 0.6 + 0.4 * frac, 0.95);

            if extraction.channel_timer.just_finished() {
                // 撤离成功！
                info!("Player successfully extracted via mining cart!");
                let mut items = Vec::new();
                if let Ok(inv) = inv_query.get_single() {
                    items.extend(inv.get_all_contained_entities());
                }

                extraction_events.send(ExtractionEvent::Success { items });
                commands.entity(entity).despawn_recursive();
                return;
            }
        } else {
            // 离开圈子则重置引导时间
            extraction.channel_timer.reset();
            sprite.color = Color::srgba(0.2, 0.9, 0.6, 0.85);
        }

        if extraction.countdown_timer.just_finished() {
            // 矿车离去
            info!("Extraction point expired and departed");
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// 结算管线与状态跳转系统
pub fn extraction_settlement_system(
    mut events: EventReader<ExtractionEvent>,
    mut session: ResMut<RunSessionContext>,
    mut save_data: ResMut<PersistentSaveData>,
    items_query: Query<(Entity, &ItemComponent, &ItemShape)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for event in events.read() {
        match event {
            ExtractionEvent::Success { items } => {
                session.secured = true;
                save_data.successful_extractions += 1;
                save_data.total_runs += 1;
                save_data.gold += session.gold_earned;
                save_data.soul_shards += session.soul_shards_earned;
                save_data.high_score = save_data.high_score.max(session.score);

                // 将局内带出的物品存入仓库持久化
                for &entity in items {
                    if let Ok((_, item, shape)) = items_query.get(entity) {
                        save_data.stash_items.push(StoredItemData {
                            item_id: item.item_id.clone(),
                            grid_x: 0,
                            grid_y: 0,
                            width: shape.width,
                            height: shape.height,
                            mask: shape.mask.clone(),
                            affixes: Vec::new(),
                        });
                    }
                }

                save_data.save_to_disk();
                info!("Extraction success! Transferred items to stash, returning to BaseCamp");
                next_state.set(AppState::BaseCamp);
            }
            ExtractionEvent::Death { .. } => {
                session.secured = false;
                save_data.total_runs += 1;
                // 死亡仅保留少量保底灵魂碎片
                let shards_kept = session.soul_shards_earned / 2;
                save_data.soul_shards += shards_kept;
                save_data.high_score = save_data.high_score.max(session.score);
                save_data.save_to_disk();

                info!("Player died in the run! Switching to GameOver");
                next_state.set(AppState::GameOver);
            }
        }
    }
}
