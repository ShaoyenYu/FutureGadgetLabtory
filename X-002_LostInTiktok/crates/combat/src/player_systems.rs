use crate::components::*;
use bevy::prelude::*;
use soulforge_core::events::SpawnDamageTextEvent;
use soulforge_core::resources::RunSessionContext;
use soulforge_data::loader::GameDatabase;
use soulforge_data::models::ItemType;
use soulforge_inventory::components::ItemComponent;
use soulforge_inventory::inventory::Inventory;
use soulforge_inventory::item_shape::ItemShape;

/// 玩家输入、移动与翻滚冲刺系统
pub fn player_movement_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Player, &CombatStats)>,
) {
    let Ok((mut transform, mut player, stats)) = query.get_single_mut() else {
        return;
    };
    let dt = time.delta_seconds();

    // 更新冲刺冷却
    player.dash_cooldown.tick(time.delta());

    let mut move_dir = Vec2::ZERO;
    if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
        move_dir.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
        move_dir.y -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
        move_dir.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
        move_dir.x += 1.0;
    }

    let is_moving = move_dir.length_squared() > 0.0;
    if is_moving {
        move_dir = move_dir.normalize();
    }

    // 触发翻滚冲刺 (Space)
    if keyboard_input.just_pressed(KeyCode::Space) && player.dash_cooldown.finished() && !player.is_dashing {
        player.is_dashing = true;
        player.dash_timer.reset();
        player.dash_cooldown.reset();
        player.dash_direction = if is_moving { move_dir } else { Vec2::Y };
    }

    // 冲刺状态处理
    if player.is_dashing {
        player.dash_timer.tick(time.delta());
        let dash_speed = stats.move_speed * 2.8;
        transform.translation.x += player.dash_direction.x * dash_speed * dt;
        transform.translation.y += player.dash_direction.y * dash_speed * dt;

        if player.dash_timer.finished() {
            player.is_dashing = false;
        }
    } else if is_moving {
        transform.translation.x += move_dir.x * stats.move_speed * dt;
        transform.translation.y += move_dir.y * stats.move_speed * dt;
    }
}

/// 掉落物磁吸与自动拾取结算系统
pub fn loot_pickup_system(
    time: Res<Time>,
    mut commands: Commands,
    mut player_query: Query<(Entity, &Transform, &Player, &mut CombatStats), Without<LootItem>>,
    mut loot_query: Query<(Entity, &mut Transform, &mut LootItem)>,
    mut inv_query: Query<&mut Inventory>,
    mut session: ResMut<RunSessionContext>,
    db: Res<GameDatabase>,
    mut text_events: EventWriter<SpawnDamageTextEvent>,
) {
    let Ok((_player_entity, player_tf, player, mut player_stats)) = player_query.get_single_mut() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();
    let dt = time.delta_seconds();

    for (loot_entity, mut loot_tf, mut loot) in loot_query.iter_mut() {
        let loot_pos = loot_tf.translation.truncate();
        let dist = player_pos.distance(loot_pos);

        // 磁吸范围判断
        if dist < player.pickup_radius || loot.magnet_active {
            loot.magnet_active = true;
            let to_player = (player_pos - loot_pos).normalize_or_zero();
            let fly_speed = 320.0;
            loot_tf.translation.x += to_player.x * fly_speed * dt;
            loot_tf.translation.y += to_player.y * fly_speed * dt;

            // 拾取距离 (< 16px)
            if dist < 16.0 {
                let item_id = &loot.item_id;

                // 特殊消耗品直接生效
                if item_id == "potion_healing" {
                    let heal = 40.0;
                    player_stats.current_hp = (player_stats.current_hp + heal).min(player_stats.max_hp);
                    text_events.send(SpawnDamageTextEvent {
                        position: player_pos + Vec2::new(0.0, 16.0),
                        amount: heal,
                        is_crit: false,
                        is_bleed: false,
                        is_heal: true,
                    });
                    commands.entity(loot_entity).despawn_recursive();
                    continue;
                } else if item_id == "soul_shard" {
                    session.soul_shards_earned += 1;
                    commands.entity(loot_entity).despawn_recursive();
                    continue;
                }

                // 普通物品尝试放入背包
                if let Some(cfg) = db.items.items.get(item_id) {
                    if let Ok(mut inv) = inv_query.get_single_mut() {
                        let mut shape = ItemShape::new(cfg.shape.width, cfg.shape.height, cfg.shape.mask.clone());
                        if let Some(pos) = inv.find_available_slot_with_rotation(&mut shape) {
                            // 创建物品实体
                            let new_item_entity = commands.spawn((
                                ItemComponent {
                                    item_id: cfg.id.clone(),
                                    name: cfg.name.clone(),
                                    item_type: cfg.item_type.clone(),
                                    base_damage: cfg.base_damage,
                                    attack_rate: cfg.attack_rate,
                                    attack_range: cfg.attack_range,
                                    projectile_count: cfg.projectile_count,
                                    color_hex: cfg.color_hex.clone(),
                                    description: cfg.description.clone(),
                                    is_equipped: cfg.item_type == ItemType::Weapon,
                                    bound_to_player: false,
                                },
                                shape.clone(),
                            )).id();

                            inv.place_item(new_item_entity, &cfg.id, &shape, pos);
                            session.collected_items.push(cfg.id.clone());
                        }
                    }
                }

                commands.entity(loot_entity).despawn_recursive();
            }
        }
    }
}
