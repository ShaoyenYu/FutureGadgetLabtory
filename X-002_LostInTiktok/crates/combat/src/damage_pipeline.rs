use crate::components::*;
use bevy::prelude::*;
use rand::Rng;
use soulforge_core::events::*;
use soulforge_core::resources::RunSessionContext;
use soulforge_data::loader::GameDatabase;

/// 核心伤害结算管线系统
pub fn damage_pipeline_system(
    mut damage_events: EventReader<DamageEvent>,
    mut stats_set: ParamSet<(
        Query<(Entity, &mut CombatStats, Option<&Enemy>, Option<&Player>, &Transform)>,
        Query<(&mut CombatStats, &Player)>,
    )>,
    affixes_query: Query<&Affixes>,
    mut text_events: EventWriter<SpawnDamageTextEvent>,
    mut loot_events: EventWriter<LootDropEvent>,
    mut reward_events: EventWriter<KillRewardEvent>,
    mut session: ResMut<RunSessionContext>,
    db: Res<GameDatabase>,
    mut commands: Commands,
) {
    let mut rng = rand::thread_rng();
    let mut pending_heals = Vec::new();

    for event in damage_events.read() {
        // 1. 获取来源词条修饰
        let mut flat_dmg: f32 = 0.0;
        let mut mult_dmg: f32 = 0.0;
        let mut lifesteal_ratio: f32 = 0.0;
        let mut bleed_chance: f32 = 0.0;
        let mut bleed_dps: f32 = 0.0;
        let mut bleed_duration: f32 = 0.0;

        if let Ok(affixes) = affixes_query.get(event.source) {
            for mod_item in &affixes.mods {
                match mod_item {
                    AffixModifier::FlatDamage(v) => flat_dmg += v,
                    AffixModifier::MultiplierDamage(v) => mult_dmg += v,
                    AffixModifier::LifeSteal(v) => lifesteal_ratio += v,
                    AffixModifier::Bleed { chance, dps, duration } => {
                        bleed_chance = bleed_chance.max(*chance);
                        bleed_dps = bleed_dps.max(*dps);
                        bleed_duration = bleed_duration.max(*duration);
                    }
                    _ => {}
                }
            }
        }

        // 2. 结算目标伤害
        let mut target_query = stats_set.p0();
        let Ok((target_entity, mut target_stats, maybe_enemy, maybe_player, transform)) =
            target_query.get_mut(event.target)
        else {
            continue;
        };

        let armor = target_stats.armor.max(0.0);
        let mitigation = armor / (armor + 50.0); // 护甲免伤公式

        let raw_damage = (event.base_damage + flat_dmg) * (1.0 + mult_dmg);
        let mut final_damage = (raw_damage * (1.0 - mitigation)).max(1.0);

        let mut is_crit = event.is_crit;
        if !is_crit && rng.gen_bool(0.15) {
            final_damage *= 1.5;
            is_crit = true;
        }

        target_stats.current_hp = (target_stats.current_hp - final_damage).max(0.0);
        let target_pos = transform.translation.truncate();

        text_events.send(SpawnDamageTextEvent {
            position: target_pos + Vec2::new(rng.gen_range(-8.0..8.0), rng.gen_range(8.0..16.0)),
            amount: final_damage,
            is_crit,
            is_bleed: event.is_bleed,
            is_heal: false,
        });

        // 记录吸血
        if lifesteal_ratio > 0.0 && maybe_enemy.is_some() {
            let heal_amount = final_damage * lifesteal_ratio;
            pending_heals.push((target_pos, heal_amount));
        }

        // 挂载流血 DOT
        if bleed_chance > 0.0 && rng.gen_bool(bleed_chance.min(1.0) as f64) && target_stats.current_hp > 0.0 {
            commands.entity(target_entity).insert(BleedStatus {
                dps: bleed_dps.max(5.0),
                remaining_time: bleed_duration.max(3.0),
                tick_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
                source: event.source,
            });
        }

        // 死亡判定
        if target_stats.current_hp <= 0.0 {
            if let Some(enemy) = maybe_enemy {
                session.kills += 1;
                session.score += (enemy.exp_reward * 10) + (enemy.gold_reward * 5);
                session.gold_earned += enemy.gold_reward;
                session.soul_shards_earned += if enemy.is_boss { 10 } else if rng.gen_bool(0.1) { 1 } else { 0 };

                reward_events.send(KillRewardEvent {
                    position: target_pos,
                    exp: enemy.exp_reward,
                    gold: enemy.gold_reward,
                    soul_shards: if enemy.is_boss { 10 } else { 0 },
                });

                // 掉落物
                if let Some(enemy_cfg) = db.enemies.enemies.get(&enemy.id) {
                    for (item_id, weight) in &enemy_cfg.drop_table {
                        if rng.gen_range(0..100) < *weight {
                            loot_events.send(LootDropEvent {
                                item_id: item_id.clone(),
                                world_position: target_pos + Vec2::new(rng.gen_range(-12.0..12.0), rng.gen_range(-12.0..12.0)),
                            });
                        }
                    }
                }

                commands.entity(target_entity).despawn_recursive();
            } else if maybe_player.is_some() {
                info!("Player has fallen in combat!");
            }
        }
    }

    // 3. 执行吸血生命恢复
    if !pending_heals.is_empty() {
        let mut player_query = stats_set.p1();
        if let Ok((mut player_stats, _)) = player_query.get_single_mut() {
            for (pos, heal_amount) in pending_heals {
                let prev_hp = player_stats.current_hp;
                player_stats.current_hp = (player_stats.current_hp + heal_amount).min(player_stats.max_hp);
                let actual_heal = player_stats.current_hp - prev_hp;
                if actual_heal > 0.5 {
                    text_events.send(SpawnDamageTextEvent {
                        position: pos + Vec2::new(0.0, 20.0),
                        amount: actual_heal,
                        is_crit: false,
                        is_bleed: false,
                        is_heal: true,
                    });
                }
            }
        }
    }
}

/// 流血状态每秒伤害跳动系统
pub fn bleed_tick_system(
    time: Res<Time>,
    mut commands: Commands,
    mut bleed_query: Query<(Entity, &mut BleedStatus, &mut CombatStats, &Transform)>,
    mut text_events: EventWriter<SpawnDamageTextEvent>,
    mut session: ResMut<RunSessionContext>,
) {
    let mut rng = rand::thread_rng();

    for (entity, mut bleed, mut stats, transform) in bleed_query.iter_mut() {
        bleed.remaining_time -= time.delta_seconds();
        bleed.tick_timer.tick(time.delta());

        if bleed.tick_timer.just_finished() {
            let tick_dmg = bleed.dps * 0.5;
            stats.current_hp = (stats.current_hp - tick_dmg).max(0.0);

            text_events.send(SpawnDamageTextEvent {
                position: transform.translation.truncate() + Vec2::new(rng.gen_range(-6.0..6.0), 12.0),
                amount: tick_dmg,
                is_crit: false,
                is_bleed: true,
                is_heal: false,
            });

            if stats.current_hp <= 0.0 {
                session.kills += 1;
                commands.entity(entity).despawn_recursive();
                continue;
            }
        }

        if bleed.remaining_time <= 0.0 {
            commands.entity(entity).remove::<BleedStatus>();
        }
    }
}

/// 纯函数伤害公式计算器，供快速单测与判定
pub fn calculate_damage_value(
    base: f32,
    flat: f32,
    multiplier: f32,
    armor: f32,
) -> f32 {
    let armor = armor.max(0.0);
    let mitigation = armor / (armor + 50.0);
    let raw = (base + flat) * (1.0 + multiplier);
    (raw * (1.0 - mitigation)).max(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_damage_formula() {
        assert_eq!(calculate_damage_value(100.0, 0.0, 0.0, 0.0), 100.0);
        assert_eq!(calculate_damage_value(100.0, 20.0, 0.5, 0.0), 180.0);
        assert_eq!(calculate_damage_value(100.0, 0.0, 0.0, 50.0), 50.0);
    }
}
