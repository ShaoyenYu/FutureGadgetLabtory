use crate::components::*;
use crate::inventory::Inventory;
use crate::item_shape::ItemShape;
use bevy::prelude::*;
use soulforge_core::events::InventoryMoveEvent;

/// 处理背包移动/放置/丢弃事件的系统
pub fn inventory_movement_handler_system(
    mut move_events: EventReader<InventoryMoveEvent>,
    mut inventory_query: Query<(Entity, &mut Inventory)>,
    mut item_query: Query<(&mut ItemComponent, &ItemShape, Option<&mut ItemGridPosition>)>,
    mut commands: Commands,
) {
    for event in move_events.read() {
        let Ok((mut item, shape, maybe_grid_pos)) = item_query.get_mut(event.item_entity) else {
            continue;
        };

        if let Ok((_inv_entity, mut inventory)) = inventory_query.get_single_mut() {
            // 1. 如果有原位置，先尝试从旧格子拔出
            if event.source_pos.is_some() {
                inventory.remove_item(event.item_entity);
            }

            // 2. 如果目标位置为 None，代表丢弃或移出背包
            if let Some(target_pos) = event.target_pos {
                if inventory.place_item(event.item_entity, &item.item_id, shape, target_pos) {
                    // 放置成功，更新或插入 ItemGridPosition
                    if let Some(mut pos) = maybe_grid_pos {
                        pos.grid_x = target_pos.0;
                        pos.grid_y = target_pos.1;
                    } else {
                        commands.entity(event.item_entity).insert(ItemGridPosition {
                            grid_x: target_pos.0,
                            grid_y: target_pos.1,
                            is_in_base_stash: false,
                        });
                    }

                    // 检查魔法格 Buff
                    let magic_buffs = inventory.get_magic_buffs_under_item(target_pos, shape);
                    if !magic_buffs.is_empty() {
                        debug!("Item {} received magic buffs: {:?}", item.name, magic_buffs);
                    }
                } else {
                    // 放置失败，若有旧位置则还原
                    if let Some(src) = event.source_pos {
                        inventory.place_item(event.item_entity, &item.item_id, shape, src);
                    }
                }
            } else {
                // 丢弃到地面或移出背包
                commands.entity(event.item_entity).remove::<ItemGridPosition>();
                item.is_equipped = false;
            }
        }
    }
}
