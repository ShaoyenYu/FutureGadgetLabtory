use bevy::prelude::*;
use soulforge_combat::components::{CombatStats, Player};
use soulforge_core::resources::{ExtractionPoint, RunSessionContext, RunTimer};

#[derive(Component)]
pub struct InRunHudRoot;

#[derive(Component)]
pub struct HpBarFill;

#[derive(Component)]
pub struct HpText;

#[derive(Component)]
pub struct WaveTimerText;

#[derive(Component)]
pub struct StatsSummaryText;

#[derive(Component)]
pub struct ExtractionRadarText;

pub fn setup_hud_system(mut commands: Commands) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    padding: UiRect::all(Val::Px(16.0)),
                    ..default()
                },
                ..default()
            },
            InRunHudRoot,
        ))
        .with_children(|root| {
            // 顶栏 HUD (左：血条；中：生存时间；右：击杀与收益)
            root.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::FlexStart,
                    ..default()
                },
                ..default()
            })
            .with_children(|top_row| {
                // 1. 生命值与翻滚状态
                top_row
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(4.0),
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|hp_box| {
                        hp_box.spawn(TextBundle::from_section(
                            "主角生命值 (HP)",
                            TextStyle {
                                font_size: 13.0,
                                color: Color::srgb(0.9, 0.9, 0.9),
                                ..default()
                            },
                        ));

                        // 外部血条框
                        hp_box
                            .spawn(NodeBundle {
                                style: Style {
                                    width: Val::Px(220.0),
                                    height: Val::Px(18.0),
                                    border: UiRect::all(Val::Px(1.0)),
                                    padding: UiRect::all(Val::Px(2.0)),
                                    ..default()
                                },
                                border_color: Color::srgb(0.4, 0.45, 0.55).into(),
                                background_color: Color::srgb(0.12, 0.14, 0.18).into(),
                                ..default()
                            })
                            .with_children(|bar_outer| {
                                bar_outer.spawn((
                                    NodeBundle {
                                        style: Style {
                                            width: Val::Percent(100.0),
                                            height: Val::Percent(100.0),
                                            ..default()
                                        },
                                        background_color: Color::srgb(0.9, 0.2, 0.25).into(),
                                        ..default()
                                    },
                                    HpBarFill,
                                ));
                            });

                        hp_box.spawn((
                            TextBundle::from_section(
                                "100 / 100",
                                TextStyle {
                                    font_size: 12.0,
                                    color: Color::srgb(0.85, 0.9, 0.95),
                                    ..default()
                                },
                            ),
                            HpText,
                        ));
                    });

                // 2. 居中生存时间与波次
                top_row
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: Val::Px(4.0),
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|center_box| {
                        center_box.spawn((
                            TextBundle::from_section(
                                "00:00",
                                TextStyle {
                                    font_size: 28.0,
                                    color: Color::srgb(0.95, 0.85, 0.3),
                                    ..default()
                                },
                            ),
                            WaveTimerText,
                        ));

                        center_box.spawn((
                            TextBundle::from_section(
                                "",
                                TextStyle {
                                    font_size: 14.0,
                                    color: Color::srgb(0.2, 0.9, 0.6),
                                    ..default()
                                },
                            ),
                            ExtractionRadarText,
                        ));
                    });

                // 3. 右侧击杀数与战利品收益
                top_row.spawn((
                    TextBundle::from_section(
                        "击杀: 0  |  金币: 0  |  灵魂碎片: 0",
                        TextStyle {
                            font_size: 14.0,
                            color: Color::srgb(0.4, 0.85, 0.95),
                            ..default()
                        },
                    ),
                    StatsSummaryText,
                ));
            });

            // 底部操作提示
            root.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            })
            .with_children(|bottom_bar| {
                bottom_bar.spawn(TextBundle::from_section(
                    "[WASD/方向键] 移动  |  [SPACE] 翻滚冲刺  |  [TAB / I] 开启网格背包 & 熔铸词条",
                    TextStyle {
                        font_size: 13.0,
                        color: Color::srgb(0.7, 0.75, 0.8),
                        ..default()
                    },
                ));
            });
        });
}

pub fn update_hud_system(
    run_timer: Res<RunTimer>,
    session: Res<RunSessionContext>,
    player_query: Query<(&CombatStats, &Transform), With<Player>>,
    extraction_query: Query<(&ExtractionPoint, &Transform), Without<Player>>,
    mut hp_bar_query: Query<&mut Style, With<HpBarFill>>,
    mut hp_text_query: Query<&mut Text, (With<HpText>, Without<WaveTimerText>, Without<StatsSummaryText>, Without<ExtractionRadarText>)>,
    mut timer_text_query: Query<&mut Text, (With<WaveTimerText>, Without<HpText>, Without<StatsSummaryText>, Without<ExtractionRadarText>)>,
    mut stats_text_query: Query<&mut Text, (With<StatsSummaryText>, Without<HpText>, Without<WaveTimerText>, Without<ExtractionRadarText>)>,
    mut radar_text_query: Query<&mut Text, (With<ExtractionRadarText>, Without<HpText>, Without<WaveTimerText>, Without<StatsSummaryText>)>,
) {
    let Ok((player_stats, player_tf)) = player_query.get_single() else {
        return;
    };

    // 1. 更新血条与血量文字
    let hp_pct = (player_stats.current_hp / player_stats.max_hp.max(1.0)).clamp(0.0, 1.0);
    if let Ok(mut style) = hp_bar_query.get_single_mut() {
        style.width = Val::Percent(hp_pct * 100.0);
    }
    if let Ok(mut hp_text) = hp_text_query.get_single_mut() {
        hp_text.sections[0].value = format!("{:.0} / {:.0}", player_stats.current_hp, player_stats.max_hp);
    }

    // 2. 更新生存计时器
    if let Ok(mut timer_text) = timer_text_query.get_single_mut() {
        let mins = (run_timer.total_seconds / 60.0).floor() as u32;
        let secs = (run_timer.total_seconds % 60.0).floor() as u32;
        timer_text.sections[0].value = format!("{:02}:{:02}", mins, secs);
    }

    // 3. 更新击杀与收益统计
    if let Ok(mut stats_text) = stats_text_query.get_single_mut() {
        stats_text.sections[0].value = format!(
            "击杀: {}  |  金币: {}  |  灵魂碎片: {}",
            session.kills, session.gold_earned, session.soul_shards_earned
        );
    }

    // 4. 更新撤离矿车雷达指示
    if let Ok(mut radar_text) = radar_text_query.get_single_mut() {
        if let Ok((extraction, ext_tf)) = extraction_query.get_single() {
            let dist = player_tf.translation.truncate().distance(ext_tf.translation.truncate());
            let time_left = extraction.countdown_timer.remaining_secs();
            radar_text.sections[0].value = format!(
                "★ 撤离矿车已就位！距离: {:.0}m | 离去倒计时: {:.0}s ★",
                dist / 16.0,
                time_left
            );
        } else {
            radar_text.sections[0].value = "".to_string();
        }
    }
}

pub fn cleanup_hud_system(
    mut commands: Commands,
    query: Query<Entity, With<InRunHudRoot>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
