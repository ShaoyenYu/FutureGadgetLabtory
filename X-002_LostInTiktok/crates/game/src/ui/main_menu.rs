use bevy::prelude::*;
use soulforge_core::states::AppState;

#[derive(Component)]
pub struct MainMenuRoot;

#[derive(Component)]
pub enum MainMenuButtonAction {
    StartRun,
    GoBaseCamp,
    Quit,
}

pub fn setup_main_menu_system(mut commands: Commands) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: Val::Px(20.0),
                    ..default()
                },
                background_color: Color::srgb(0.08, 0.09, 0.12).into(),
                ..default()
            },
            MainMenuRoot,
        ))
        .with_children(|parent| {
            // 游戏主标题
            parent.spawn(TextBundle::from_section(
                "SOULFORGE: LOST IN TIKTOK",
                TextStyle {
                    font_size: 42.0,
                    color: Color::srgb(0.95, 0.77, 0.25),
                    ..default()
                },
            ));

            // 副标题
            parent.spawn(TextBundle::from_section(
                "Survivor + Grid Inventory + Extraction + Weapon Forging",
                TextStyle {
                    font_size: 18.0,
                    color: Color::srgb(0.7, 0.75, 0.8),
                    ..default()
                },
            ));

            // 按钮容器
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(12.0),
                        margin: UiRect::top(Val::Px(24.0)),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|btn_parent| {
                    spawn_menu_button(btn_parent, "开启深渊探险 (Start Run)", MainMenuButtonAction::StartRun);
                    spawn_menu_button(btn_parent, "安全屋 / 铁匠铺 (Base Camp)", MainMenuButtonAction::GoBaseCamp);
                    spawn_menu_button(btn_parent, "退出游戏 (Quit)", MainMenuButtonAction::Quit);
                });
        });
}

fn spawn_menu_button(
    parent: &mut ChildBuilder,
    label: &str,
    action: MainMenuButtonAction,
) {
    parent
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(280.0),
                    height: Val::Px(48.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                border_color: Color::srgb(0.4, 0.45, 0.55).into(),
                background_color: Color::srgb(0.18, 0.2, 0.26).into(),
                ..default()
            },
            action,
        ))
        .with_children(|btn| {
            btn.spawn(TextBundle::from_section(
                label,
                TextStyle {
                    font_size: 16.0,
                    color: Color::srgb(0.95, 0.95, 0.95),
                    ..default()
                },
            ));
        });
}

pub fn main_menu_interaction_system(
    mut interaction_query: Query<
        (&Interaction, &MainMenuButtonAction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<AppState>>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    for (interaction, action, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => match action {
                MainMenuButtonAction::StartRun => {
                    next_state.set(AppState::RunSpawning);
                }
                MainMenuButtonAction::GoBaseCamp => {
                    next_state.set(AppState::BaseCamp);
                }
                MainMenuButtonAction::Quit => {
                    app_exit_events.send(AppExit::Success);
                }
            },
            Interaction::Hovered => {
                *bg_color = Color::srgb(0.28, 0.32, 0.42).into();
            }
            Interaction::None => {
                *bg_color = Color::srgb(0.18, 0.2, 0.26).into();
            }
        }
    }
}

pub fn cleanup_main_menu_system(
    mut commands: Commands,
    query: Query<Entity, With<MainMenuRoot>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
