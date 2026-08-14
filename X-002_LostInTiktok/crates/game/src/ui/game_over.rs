use bevy::prelude::*;
use soulforge_core::resources::RunSessionContext;
use soulforge_core::states::AppState;
use soulforge_meta::save_data::PersistentSaveData;

#[derive(Component)]
pub struct SettlementScreenRoot;

#[derive(Component)]
pub enum SettlementButtonAction {
    ReturnToBaseCamp,
}

pub fn setup_game_over_settlement_system(
    mut commands: Commands,
    session: Res<RunSessionContext>,
    save_data: Res<PersistentSaveData>,
) {
    let is_victory = session.secured;
    let title_text = if is_victory {
        "★ 成功搭乘矿车撤离 (EXTRACTION SUCCESSFUL) ★"
    } else {
        "☠ 魂飞魄散 (DEFEATED IN THE ABYSS) ☠"
    };

    let title_color = if is_victory {
        Color::srgb(0.2, 0.95, 0.5)
    } else {
        Color::srgb(0.95, 0.25, 0.25)
    };

    let mins = (session.time_survived / 60.0).floor() as u32;
    let secs = (session.time_survived % 60.0).floor() as u32;

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(16.0),
                    ..default()
                },
                background_color: Color::srgba(0.05, 0.06, 0.08, 0.95).into(),
                ..default()
            },
            SettlementScreenRoot,
        ))
        .with_children(|root| {
            // 标题
            root.spawn(TextBundle::from_section(
                title_text,
                TextStyle {
                    font_size: 32.0,
                    color: title_color,
                    ..default()
                },
            ));

            // 战报统计面板
            root.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(420.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(16.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    row_gap: Val::Px(10.0),
                    margin: UiRect::axes(Val::Px(0.0), Val::Px(16.0)),
                    ..default()
                },
                border_color: Color::srgb(0.4, 0.45, 0.55).into(),
                background_color: Color::srgb(0.12, 0.14, 0.18).into(),
                ..default()
            })
            .with_children(|panel| {
                panel.spawn(TextBundle::from_section(
                    format!("• 存活时间: {:02}:{:02}", mins, secs),
                    TextStyle {
                        font_size: 15.0,
                        color: Color::srgb(0.9, 0.9, 0.9),
                        ..default()
                    },
                ));

                panel.spawn(TextBundle::from_section(
                    format!("• 击杀怪物总数: {}", session.kills),
                    TextStyle {
                        font_size: 15.0,
                        color: Color::srgb(0.9, 0.9, 0.9),
                        ..default()
                    },
                ));

                panel.spawn(TextBundle::from_section(
                    format!("• 搜刮带出战利品: {} 件 (已入避难所仓库)", session.collected_items.len()),
                    TextStyle {
                        font_size: 15.0,
                        color: Color::srgb(0.3, 0.85, 0.95),
                        ..default()
                    },
                ));

                panel.spawn(TextBundle::from_section(
                    format!("• 本局金币收益: +{} G  |  灵魂碎片: +{}", session.gold_earned, session.soul_shards_earned),
                    TextStyle {
                        font_size: 15.0,
                        color: Color::srgb(0.95, 0.85, 0.3),
                        ..default()
                    },
                ));

                panel.spawn(TextBundle::from_section(
                    format!("• 避难所总资产: {} 金币 | {} 灵魂碎片", save_data.gold, save_data.soul_shards),
                    TextStyle {
                        font_size: 15.0,
                        color: Color::srgb(0.7, 0.95, 0.7),
                        ..default()
                    },
                ));
            });

            // 返回安全屋营地按钮
            root.spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(260.0),
                        height: Val::Px(48.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    border_color: Color::srgb(0.95, 0.77, 0.25).into(),
                    background_color: Color::srgb(0.2, 0.25, 0.35).into(),
                    ..default()
                },
                SettlementButtonAction::ReturnToBaseCamp,
            ))
            .with_children(|btn| {
                btn.spawn(TextBundle::from_section(
                    "返回安全屋营地 (Return to Base Camp)",
                    TextStyle {
                        font_size: 15.0,
                        color: Color::srgb(1.0, 1.0, 1.0),
                        ..default()
                    },
                ));
            });
        });
}

pub fn settlement_interaction_system(
    mut interaction_query: Query<
        (&Interaction, &SettlementButtonAction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, action, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => match action {
                SettlementButtonAction::ReturnToBaseCamp => {
                    next_state.set(AppState::BaseCamp);
                }
            },
            Interaction::Hovered => {
                *bg_color = Color::srgb(0.32, 0.38, 0.5).into();
            }
            Interaction::None => {
                *bg_color = Color::srgb(0.2, 0.25, 0.35).into();
            }
        }
    }
}

pub fn cleanup_settlement_system(
    mut commands: Commands,
    query: Query<Entity, With<SettlementScreenRoot>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
