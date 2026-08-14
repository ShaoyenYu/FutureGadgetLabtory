use bevy::prelude::*;
use soulforge_core::states::AppState;
use soulforge_data::loader::GameDatabase;
use soulforge_data::models::ItemType;
use soulforge_meta::save_data::PersistentSaveData;

#[derive(Component)]
pub struct BaseCampRoot;

#[derive(Component)]
pub enum BaseCampButtonAction {
    StartRun,
    BackToMenu,
    UpgradeTalent(String),
    ForgeSelectedRelic(usize),
}

pub fn setup_base_camp_system(
    mut commands: Commands,
    save_data: Res<PersistentSaveData>,
    db: Res<GameDatabase>,
) {
    build_base_camp_ui(&mut commands, &save_data, &db);
}

pub fn build_base_camp_ui(
    commands: &mut Commands,
    save_data: &PersistentSaveData,
    db: &GameDatabase,
) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    padding: UiRect::all(Val::Px(20.0)),
                    ..default()
                },
                background_color: Color::srgb(0.09, 0.1, 0.14).into(),
                ..default()
            },
            BaseCampRoot,
        ))
        .with_children(|root| {
            // 顶栏：安全屋标题与资源货币
            root.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    padding: UiRect::bottom(Val::Px(12.0)),
                    border: UiRect::bottom(Val::Px(1.0)),
                    ..default()
                },
                border_color: Color::srgb(0.25, 0.3, 0.4).into(),
                ..default()
            })
            .with_children(|header| {
                header.spawn(TextBundle::from_section(
                    "安全屋营地 (BASE CAMP & BLACKSMITH)",
                    TextStyle {
                        font_size: 24.0,
                        color: Color::srgb(0.95, 0.77, 0.25),
                        ..default()
                    },
                ));

                header.spawn(TextBundle::from_section(
                    format!("金币: {}  |  纯净灵魂碎片: {}  |  最高积分: {}", save_data.gold, save_data.soul_shards, save_data.high_score),
                    TextStyle {
                        font_size: 16.0,
                        color: Color::srgb(0.3, 0.85, 0.95),
                        ..default()
                    },
                ));
            });

            // 中间主要功能区 (3列：铁匠铺锻造、安全仓库、天赋神坛)
            root.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(78.0),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(16.0),
                    ..default()
                },
                ..default()
            })
            .with_children(|main_cols| {
                // 1. 铁匠铺专区
                main_cols.spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(33.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(14.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        row_gap: Val::Px(10.0),
                        ..default()
                    },
                    border_color: Color::srgb(0.6, 0.35, 0.15).into(),
                    background_color: Color::srgb(0.13, 0.14, 0.18).into(),
                    ..default()
                })
                .with_children(|forge_col| {
                    forge_col.spawn(TextBundle::from_section(
                        "铁匠铺 (Forge Relics)",
                        TextStyle {
                            font_size: 18.0,
                            color: Color::srgb(0.95, 0.65, 0.2),
                            ..default()
                        },
                    ));

                    let starting_weapon_name = db.items.items.get(&save_data.starting_weapon_id)
                        .map(|i| i.name.as_str()).unwrap_or("破败的剑胚");

                    forge_col.spawn(TextBundle::from_section(
                        format!("当前主武器: {}\n可在下方消耗仓库圣骸为其熔铸强力随机词条：", starting_weapon_name),
                        TextStyle {
                            font_size: 13.0,
                            color: Color::srgb(0.8, 0.8, 0.8),
                            ..default()
                        },
                    ));

                    // 遍历仓库中可用于锻造的圣骸材料
                    for (idx, stored_item) in save_data.stash_items.iter().enumerate() {
                        if let Some(item_cfg) = db.items.items.get(&stored_item.item_id) {
                            if item_cfg.item_type == ItemType::Material && !item_cfg.possible_affixes.is_empty() {
                                forge_col.spawn((
                                    ButtonBundle {
                                        style: Style {
                                            width: Val::Percent(100.0),
                                            height: Val::Px(36.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border: UiRect::all(Val::Px(1.0)),
                                            margin: UiRect::top(Val::Px(4.0)),
                                            ..default()
                                        },
                                        border_color: Color::srgb(0.7, 0.4, 0.2).into(),
                                        background_color: Color::srgb(0.22, 0.18, 0.15).into(),
                                        ..default()
                                    },
                                    BaseCampButtonAction::ForgeSelectedRelic(idx),
                                ))
                                .with_children(|btn| {
                                    btn.spawn(TextBundle::from_section(
                                        format!("熔铸: {} (抽取词条)", item_cfg.name),
                                        TextStyle {
                                            font_size: 13.0,
                                            color: Color::srgb(0.95, 0.85, 0.6),
                                            ..default()
                                        },
                                    ));
                                });
                            }
                        }
                    }
                });

                // 2. 仓库物资专区
                main_cols.spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(33.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(14.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        row_gap: Val::Px(8.0),
                        ..default()
                    },
                    border_color: Color::srgb(0.25, 0.35, 0.5).into(),
                    background_color: Color::srgb(0.12, 0.14, 0.2).into(),
                    ..default()
                })
                .with_children(|stash_col| {
                    stash_col.spawn(TextBundle::from_section(
                        format!("避难所仓库 (已存 {} 件战利品)", save_data.stash_items.len()),
                        TextStyle {
                            font_size: 18.0,
                            color: Color::srgb(0.4, 0.75, 0.95),
                            ..default()
                        },
                    ));

                    for stored in &save_data.stash_items {
                        if let Some(item_cfg) = db.items.items.get(&stored.item_id) {
                            stash_col.spawn(TextBundle::from_section(
                                format!("• {} [{}x{}]", item_cfg.name, stored.width, stored.height),
                                TextStyle {
                                    font_size: 13.0,
                                    color: Color::srgb(0.85, 0.9, 0.95),
                                    ..default()
                                },
                            ));
                        }
                    }
                });

                // 3. 天赋神坛专区
                main_cols.spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(33.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(14.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        row_gap: Val::Px(8.0),
                        ..default()
                    },
                    border_color: Color::srgb(0.45, 0.25, 0.6).into(),
                    background_color: Color::srgb(0.15, 0.12, 0.22).into(),
                    ..default()
                })
                .with_children(|talent_col| {
                    talent_col.spawn(TextBundle::from_section(
                        "天赋神坛 (Meta Talents)",
                        TextStyle {
                            font_size: 18.0,
                            color: Color::srgb(0.85, 0.55, 0.95),
                            ..default()
                        },
                    ));

                    for talent in &db.talents.talents {
                        let cur_lvl = save_data.talent_levels.get(&talent.id).copied().unwrap_or(0);
                        let cost = (talent.cost_base as f32 * talent.cost_multiplier.powi(cur_lvl as i32)) as u32;

                        talent_col.spawn((
                            ButtonBundle {
                                style: Style {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(42.0),
                                    justify_content: JustifyContent::SpaceBetween,
                                    align_items: AlignItems::Center,
                                    padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    margin: UiRect::top(Val::Px(4.0)),
                                    ..default()
                                },
                                border_color: Color::srgb(0.55, 0.35, 0.75).into(),
                                background_color: Color::srgb(0.24, 0.18, 0.32).into(),
                                ..default()
                            },
                            BaseCampButtonAction::UpgradeTalent(talent.id.clone()),
                        ))
                        .with_children(|btn| {
                            btn.spawn(TextBundle::from_section(
                                format!("{}: Lv.{}/{}", talent.name, cur_lvl, talent.max_level),
                                TextStyle {
                                    font_size: 13.0,
                                    color: Color::srgb(0.95, 0.95, 0.95),
                                    ..default()
                                },
                            ));

                            let cost_text = if cur_lvl >= talent.max_level {
                                "已满级".to_string()
                            } else {
                                format!("升级: {}G", cost)
                            };

                            btn.spawn(TextBundle::from_section(
                                cost_text,
                                TextStyle {
                                    font_size: 12.0,
                                    color: Color::srgb(0.95, 0.8, 0.3),
                                    ..default()
                                },
                            ));
                        });
                    }
                });
            });

            // 底栏操作按钮
            root.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            })
            .with_children(|footer| {
                footer.spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(180.0),
                            height: Val::Px(44.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        border_color: Color::srgb(0.4, 0.45, 0.55).into(),
                        background_color: Color::srgb(0.18, 0.2, 0.26).into(),
                        ..default()
                    },
                    BaseCampButtonAction::BackToMenu,
                ))
                .with_children(|btn| {
                    btn.spawn(TextBundle::from_section(
                        "返回主菜单 (Menu)",
                        TextStyle {
                            font_size: 15.0,
                            color: Color::srgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });

                footer.spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(240.0),
                            height: Val::Px(46.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        border_color: Color::srgb(0.9, 0.7, 0.2).into(),
                        background_color: Color::srgb(0.6, 0.35, 0.1).into(),
                        ..default()
                    },
                    BaseCampButtonAction::StartRun,
                ))
                .with_children(|btn| {
                    btn.spawn(TextBundle::from_section(
                        "进入深渊探险 (Start Run)",
                        TextStyle {
                            font_size: 16.0,
                            color: Color::srgb(1.0, 0.95, 0.8),
                            ..default()
                        },
                    ));
                });
            });
        });
}

pub fn base_camp_interaction_system(
    mut interaction_query: Query<
        (&Interaction, &BaseCampButtonAction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut save_data: ResMut<PersistentSaveData>,
    db: Res<GameDatabase>,
    mut next_state: ResMut<NextState<AppState>>,
    mut commands: Commands,
    camp_query: Query<Entity, With<BaseCampRoot>>,
) {
    let mut need_refresh = false;

    for (interaction, action, mut bg_color) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            match action {
                BaseCampButtonAction::StartRun => {
                    next_state.set(AppState::RunSpawning);
                }
                BaseCampButtonAction::BackToMenu => {
                    next_state.set(AppState::MainMenu);
                }
                BaseCampButtonAction::UpgradeTalent(talent_id) => {
                    if save_data.try_upgrade_talent(talent_id, &db) {
                        info!("Upgraded talent {}", talent_id);
                        need_refresh = true;
                    }
                }
                BaseCampButtonAction::ForgeSelectedRelic(stash_idx) => {
                    if *stash_idx < save_data.stash_items.len() {
                        let consumed = save_data.stash_items.remove(*stash_idx);
                        info!("Consumed relic {} to forge starting weapon", consumed.item_id);
                        save_data.save_to_disk();
                        need_refresh = true;
                    }
                }
            }
        } else if *interaction == Interaction::Hovered {
            *bg_color = Color::srgb(0.35, 0.4, 0.52).into();
        } else {
            *bg_color = Color::srgb(0.18, 0.2, 0.26).into();
        }
    }

    // 若进行升级或锻造，刷新 UI 界面
    if need_refresh {
        for entity in camp_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        build_base_camp_ui(&mut commands, &save_data, &db);
    }
}

pub fn cleanup_base_camp_system(
    mut commands: Commands,
    query: Query<Entity, With<BaseCampRoot>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
