use bevy::prelude::*;
use soulforge_combat::components::{AffixModifier, Affixes, CombatStats};
use soulforge_core::events::InventoryMoveEvent;
use soulforge_core::resources::GamePaused;
use soulforge_data::models::ItemType;
use soulforge_inventory::components::ItemComponent;
use soulforge_inventory::inventory::Inventory;
use soulforge_inventory::item_shape::ItemShape;

/// 记录当前选中的物品（用于旋转、移动、锻造）
#[derive(Resource, Debug, Clone, Default)]
pub struct SelectedInventoryItem {
    pub entity: Option<Entity>,
    pub item_id: Option<String>,
    pub temp_shape: Option<ItemShape>,
    pub original_pos: Option<(u8, u8)>,
}

#[derive(Component)]
pub struct InventoryModalRoot;

#[derive(Component)]
pub struct InventoryCellButton {
    pub x: u8,
    pub y: u8,
}

#[derive(Component)]
pub struct ItemCardButton {
    pub entity: Entity,
}

/// 监听 Tab / I 键开关网格背包
pub fn toggle_inventory_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut paused: ResMut<GamePaused>,
    modal_query: Query<Entity, With<InventoryModalRoot>>,
    mut commands: Commands,
    player_query: Query<(&Inventory, &CombatStats)>,
    item_query: Query<(Entity, &ItemComponent, &ItemShape, Option<&Affixes>)>,
    selected: Res<SelectedInventoryItem>,
) {
    if keyboard_input.just_pressed(KeyCode::Tab) || keyboard_input.just_pressed(KeyCode::KeyI) {
        if modal_query.is_empty() {
            // 打开背包：暂停游戏逻辑并渲染背包 UI
            paused.0 = true;
            setup_inventory_modal(&mut commands, &player_query, &item_query, &selected);
        } else {
            // 关闭背包：恢复游戏
            paused.0 = false;
            for entity in modal_query.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

/// 监听 R 键旋转当前选中的物品
pub fn rotate_selected_item_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut selected: ResMut<SelectedInventoryItem>,
    modal_query: Query<Entity, With<InventoryModalRoot>>,
    player_query: Query<(&Inventory, &CombatStats)>,
    mut item_query: Query<(Entity, &ItemComponent, &mut ItemShape, Option<&Affixes>)>,
    mut commands: Commands,
) {
    if keyboard_input.just_pressed(KeyCode::KeyR) && !modal_query.is_empty() {
        if let Some(entity) = selected.entity {
            if let Ok((_, _, mut shape, _)) = item_query.get_mut(entity) {
                shape.rotate_90();
                selected.temp_shape = Some(shape.clone());
                info!("Rotated selected item by 90 degrees: new dimensions {}x{}", shape.width, shape.height);

                // 重新渲染背包界面
                for m in modal_query.iter() {
                    commands.entity(m).despawn_recursive();
                }

                // 准备只读视图用于重新渲染
                let read_only_items: Vec<(Entity, ItemComponent, ItemShape, Option<Affixes>)> = item_query
                    .iter()
                    .map(|(e, c, s, a)| (e, c.clone(), s.clone(), a.cloned()))
                    .collect();

                rebuild_inventory_modal_from_data(&mut commands, &player_query, &read_only_items, &selected);
            }
        }
    }
}

/// 渲染网格背包弹窗界面（基于 Query）
pub fn setup_inventory_modal(
    commands: &mut Commands,
    player_query: &Query<(&Inventory, &CombatStats)>,
    item_query: &Query<(Entity, &ItemComponent, &ItemShape, Option<&Affixes>)>,
    selected: &SelectedInventoryItem,
) {
    let items_data: Vec<(Entity, ItemComponent, ItemShape, Option<Affixes>)> = item_query
        .iter()
        .map(|(e, c, s, a)| (e, c.clone(), s.clone(), a.cloned()))
        .collect();

    rebuild_inventory_modal_from_data(commands, player_query, &items_data, selected);
}

/// 渲染网格背包弹窗界面（核心通用构建器）
pub fn rebuild_inventory_modal_from_data(
    commands: &mut Commands,
    player_query: &Query<(&Inventory, &CombatStats)>,
    items: &[(Entity, ItemComponent, ItemShape, Option<Affixes>)],
    selected: &SelectedInventoryItem,
) {
    let Ok((inventory, stats)) = player_query.get_single() else {
        return;
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::srgba(0.04, 0.05, 0.08, 0.88).into(),
                ..default()
            },
            InventoryModalRoot,
        ))
        .with_children(|backdrop| {
            backdrop
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(780.0),
                        height: Val::Px(480.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(18.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        row_gap: Val::Px(12.0),
                        ..default()
                    },
                    border_color: Color::srgb(0.5, 0.55, 0.7).into(),
                    background_color: Color::srgb(0.1, 0.12, 0.16).into(),
                    ..default()
                })
                .with_children(|panel| {
                    // 1. 顶栏说明
                    panel
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                justify_content: JustifyContent::SpaceBetween,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|header| {
                            header.spawn(TextBundle::from_section(
                                "战术异形背包 (GRID INVENTORY) - [R] 旋转物品 | [TAB/I] 关闭",
                                TextStyle {
                                    font_size: 17.0,
                                    color: Color::srgb(0.95, 0.77, 0.25),
                                    ..default()
                                },
                            ));

                            header.spawn(TextBundle::from_section(
                                format!("攻击力: {:.1}  |  护甲: {:.1}  |  移速: {:.0}", stats.base_damage, stats.armor, stats.move_speed),
                                TextStyle {
                                    font_size: 13.0,
                                    color: Color::srgb(0.4, 0.85, 0.95),
                                    ..default()
                                },
                            ));
                        });

                    // 2. 主内容区 (左：网格背包；右：物品列表与词条详情)
                    panel
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                height: Val::Percent(88.0),
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(18.0),
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|content_row| {
                            // 左侧：6x4 网格渲染
                            content_row
                                .spawn(NodeBundle {
                                    style: Style {
                                        width: Val::Px(360.0),
                                        height: Val::Percent(100.0),
                                        flex_direction: FlexDirection::Column,
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        border: UiRect::all(Val::Px(1.0)),
                                        padding: UiRect::all(Val::Px(12.0)),
                                        ..default()
                                    },
                                    border_color: Color::srgb(0.3, 0.35, 0.45).into(),
                                    background_color: Color::srgb(0.06, 0.08, 0.1).into(),
                                    ..default()
                                })
                                .with_children(|grid_container| {
                                    for y in 0..inventory.max_height {
                                        grid_container
                                            .spawn(NodeBundle {
                                                style: Style {
                                                    flex_direction: FlexDirection::Row,
                                                    column_gap: Val::Px(4.0),
                                                    margin: UiRect::axes(Val::Px(0.0), Val::Px(2.0)),
                                                    ..default()
                                                },
                                                ..default()
                                            })
                                            .with_children(|row| {
                                                for x in 0..inventory.max_width {
                                                    let is_unlocked = inventory.unlocked_cells.contains(&(x, y));
                                                    let maybe_slot = inventory.slots.get(&(x, y));
                                                    let has_item = maybe_slot.and_then(|s| s.item_entity).is_some();
                                                    let is_magic = maybe_slot.and_then(|s| s.magic_buff.as_ref()).is_some();

                                                    let bg_color = if !is_unlocked {
                                                        Color::srgb(0.04, 0.04, 0.06)
                                                    } else if has_item {
                                                        Color::srgb(0.25, 0.4, 0.6)
                                                    } else {
                                                        Color::srgb(0.14, 0.16, 0.22)
                                                    };

                                                    let border_color = if is_magic {
                                                        Color::srgb(0.95, 0.77, 0.2) // 魔法格金边
                                                    } else {
                                                        Color::srgb(0.28, 0.32, 0.42)
                                                    };

                                                    row.spawn((
                                                        ButtonBundle {
                                                            style: Style {
                                                                width: Val::Px(48.0),
                                                                height: Val::Px(48.0),
                                                                border: UiRect::all(Val::Px(if is_magic { 2.0 } else { 1.0 })),
                                                                justify_content: JustifyContent::Center,
                                                                align_items: AlignItems::Center,
                                                                ..default()
                                                            },
                                                            border_color: border_color.into(),
                                                            background_color: bg_color.into(),
                                                            ..default()
                                                        },
                                                        InventoryCellButton { x, y },
                                                    ))
                                                    .with_children(|cell| {
                                                        if is_magic {
                                                            cell.spawn(TextBundle::from_section(
                                                                "★",
                                                                TextStyle {
                                                                    font_size: 10.0,
                                                                    color: Color::srgb(0.95, 0.8, 0.2),
                                                                    ..default()
                                                                },
                                                            ));
                                                        }
                                                    });
                                                }
                                            });
                                    }
                                });

                            // 右侧：物品列表与词条详情面板
                            content_row
                                .spawn(NodeBundle {
                                    style: Style {
                                        width: Val::Px(360.0),
                                        height: Val::Percent(100.0),
                                        flex_direction: FlexDirection::Column,
                                        row_gap: Val::Px(6.0),
                                        padding: UiRect::all(Val::Px(10.0)),
                                        border: UiRect::all(Val::Px(1.0)),
                                        ..default()
                                    },
                                    border_color: Color::srgb(0.3, 0.35, 0.45).into(),
                                    background_color: Color::srgb(0.08, 0.1, 0.13).into(),
                                    ..default()
                                })
                                .with_children(|details_col| {
                                    details_col.spawn(TextBundle::from_section(
                                        "背包内物品与圣骸 (点击选中/移动):",
                                        TextStyle {
                                            font_size: 14.0,
                                            color: Color::srgb(0.9, 0.9, 0.9),
                                            ..default()
                                        },
                                    ));

                                    for (item_entity, item, shape, maybe_affixes) in items {
                                        let is_selected = selected.entity == Some(*item_entity);

                                        let mut affixes_str = String::new();
                                        if let Some(affixes) = maybe_affixes {
                                            for mod_item in &affixes.mods {
                                                match mod_item {
                                                    AffixModifier::FlatDamage(v) => affixes_str.push_str(&format!(" +{:.0}基础伤", v)),
                                                    AffixModifier::MultiplierDamage(v) => affixes_str.push_str(&format!(" +{:.0}%乘伤", v * 100.0)),
                                                    AffixModifier::LifeSteal(v) => affixes_str.push_str(&format!(" +{:.0}%吸血", v * 100.0)),
                                                    AffixModifier::Pierce(v) => affixes_str.push_str(&format!(" +{}穿透", v)),
                                                    AffixModifier::Bleed { chance, .. } => affixes_str.push_str(&format!(" +{:.0}%撕裂流血", chance * 100.0)),
                                                    _ => {}
                                                }
                                            }
                                        }

                                        let type_tag = match item.item_type {
                                            ItemType::Weapon => "[武器]",
                                            ItemType::Material => "[圣骸材料]",
                                            ItemType::Consumable => "[药剂]",
                                            ItemType::Artifact => "[神器]",
                                        };

                                        details_col
                                            .spawn((
                                                ButtonBundle {
                                                    style: Style {
                                                        width: Val::Percent(100.0),
                                                        min_height: Val::Px(42.0),
                                                        flex_direction: FlexDirection::Column,
                                                        padding: UiRect::all(Val::Px(6.0)),
                                                        border: UiRect::all(Val::Px(if is_selected { 2.0 } else { 1.0 })),
                                                        margin: UiRect::bottom(Val::Px(4.0)),
                                                        ..default()
                                                    },
                                                    border_color: Color::srgb(if is_selected { 0.95 } else { 0.35 }, if is_selected { 0.77 } else { 0.4 }, 0.5).into(),
                                                    background_color: Color::srgb(if is_selected { 0.25 } else { 0.15 }, 0.18, 0.25).into(),
                                                    ..default()
                                                },
                                                ItemCardButton { entity: *item_entity },
                                            ))
                                            .with_children(|card| {
                                                card.spawn(TextBundle::from_section(
                                                    format!("{} {} ({}x{}){}", type_tag, item.name, shape.width, shape.height, if item.is_equipped { " ★已装备" } else { "" }),
                                                    TextStyle {
                                                        font_size: 12.0,
                                                        color: Color::srgb(0.95, 0.9, 0.6),
                                                        ..default()
                                                    },
                                                ));

                                                if !affixes_str.is_empty() {
                                                    card.spawn(TextBundle::from_section(
                                                        format!("圣骸词条:{}", affixes_str),
                                                        TextStyle {
                                                            font_size: 11.0,
                                                            color: Color::srgb(0.4, 0.9, 0.6),
                                                            ..default()
                                                        },
                                                    ));
                                                }
                                            });
                                    }
                                });
                        });
                });
        });
}

/// 背包单元格点击与移动交互系统
pub fn inventory_interaction_system(
    mut cell_query: Query<(&Interaction, &InventoryCellButton), (Changed<Interaction>, With<Button>)>,
    mut card_query: Query<(&Interaction, &ItemCardButton), (Changed<Interaction>, With<Button>)>,
    mut selected: ResMut<SelectedInventoryItem>,
    mut move_events: EventWriter<InventoryMoveEvent>,
    player_query: Query<(&Inventory, &CombatStats)>,
    item_query: Query<(Entity, &ItemComponent, &ItemShape, Option<&Affixes>)>,
    modal_query: Query<Entity, With<InventoryModalRoot>>,
    mut commands: Commands,
) {
    let mut need_refresh = false;

    // 1. 点击物品卡片 -> 设为选中
    for (interaction, card) in card_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            selected.entity = Some(card.entity);
            if let Ok((_, item, shape, _)) = item_query.get(card.entity) {
                selected.item_id = Some(item.item_id.clone());
                selected.temp_shape = Some(shape.clone());
                info!("Selected item for placement/rotation: {}", item.name);
                need_refresh = true;
            }
        }
    }

    // 2. 点击网格单元 -> 尝试将选中物品移动/放置到目标坐标
    for (interaction, cell) in cell_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            if let Some(selected_entity) = selected.entity {
                if let Ok((_, _, shape, _)) = item_query.get(selected_entity) {
                    if let Ok((inv, _)) = player_query.get_single() {
                        let target_pos = (cell.x, cell.y);
                        if inv.can_place_item(shape, target_pos, Some(selected_entity)) {
                            // 校验通过，派发移动事件
                            move_events.send(InventoryMoveEvent {
                                item_entity: selected_entity,
                                source_pos: selected.original_pos,
                                target_pos: Some(target_pos),
                            });
                            selected.entity = None;
                            need_refresh = true;
                        } else {
                            warn!("Cannot place item at ({}, {}): bounds/overlap conflict", cell.x, cell.y);
                        }
                    }
                }
            }
        }
    }

    if need_refresh {
        for m in modal_query.iter() {
            commands.entity(m).despawn_recursive();
        }
        setup_inventory_modal(&mut commands, &player_query, &item_query, &selected);
    }
}
