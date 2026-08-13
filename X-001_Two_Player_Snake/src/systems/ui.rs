use crate::components::*;
use bevy::prelude::*;

pub fn setup_menus(mut commands: Commands) {
    fn spawn_button(p: &mut ChildBuilder, text: &str, action: UIAction) {
        p.spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(220.0),
                    height: Val::Px(70.0),
                    margin: UiRect::all(Val::Px(15.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::srgb(0.2, 0.2, 0.2).into(),
                ..default()
            },
            action,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                text,
                TextStyle {
                    font_size: 35.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
        });
    }

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    display: Display::None,
                    ..default()
                },
                background_color: Color::srgba(0.0, 0.0, 0.0, 0.8).into(),
                z_index: ZIndex::Global(100),
                ..default()
            },
            PauseMenuUI,
        ))
        .with_children(|p| {
            p.spawn(TextBundle::from_section(
                "PAUSED",
                TextStyle {
                    font_size: 80.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
            spawn_button(p, "Resume", UIAction::Resume);
            spawn_button(p, "Settings", UIAction::Settings);
            spawn_button(p, "Restart", UIAction::Restart);
        });

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    display: Display::None,
                    ..default()
                },
                background_color: Color::srgba(0.0, 0.0, 0.0, 0.95).into(),
                z_index: ZIndex::Global(101),
                ..default()
            },
            SettingsMenuUI,
        ))
        .with_children(|p| {
            p.spawn(TextBundle::from_section(
                "SETTINGS",
                TextStyle {
                    font_size: 80.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));

            p.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    margin: UiRect::all(Val::Px(20.0)),
                    ..default()
                },
                ..default()
            })
            .with_children(|row| {
                row.spawn(TextBundle::from_section(
                    "Initial HP: ",
                    TextStyle {
                        font_size: 40.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ));
                spawn_button(row, "-", UIAction::DecreaseHP);
                row.spawn((
                    TextBundle::from_section(
                        "5",
                        TextStyle {
                            font_size: 40.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ),
                    HPText,
                ));
                spawn_button(row, "+", UIAction::IncreaseHP);
            });

            spawn_button(p, "Back", UIAction::Back);
        });
}

pub fn spawn_ui(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(100.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(50.0), Val::Px(20.0)),
                ..default()
            },
            background_color: Color::srgba(0.0, 0.0, 0.0, 0.6).into(),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        align_items: AlignItems::Center,
                        padding: UiRect::axes(Val::Px(25.0), Val::Px(10.0)),
                        ..default()
                    },
                    background_color: Color::srgba(0.2, 0.8, 1.0, 0.15).into(),
                    border_radius: BorderRadius::all(Val::Px(10.0)),
                    ..default()
                })
                .with_children(|p| {
                    p.spawn((
                        TextBundle::from_section(
                            "P2 (WASD) : 0",
                            TextStyle {
                                font_size: 35.0,
                                color: Color::srgb(0.2, 0.8, 1.0),
                                ..default()
                            },
                        ),
                        ScoreText(Player::Two),
                    ));
                });

            parent
                .spawn(NodeBundle {
                    style: Style {
                        align_items: AlignItems::Center,
                        padding: UiRect::axes(Val::Px(25.0), Val::Px(10.0)),
                        ..default()
                    },
                    background_color: Color::srgba(0.2, 1.0, 0.2, 0.15).into(),
                    border_radius: BorderRadius::all(Val::Px(10.0)),
                    ..default()
                })
                .with_children(|p| {
                    p.spawn((
                        TextBundle::from_section(
                            "P1 (Arrows) : 0",
                            TextStyle {
                                font_size: 35.0,
                                color: Color::srgb(0.2, 1.0, 0.2),
                                ..default()
                            },
                        ),
                        ScoreText(Player::One),
                    ));
                });
        });
}

pub fn update_score_ui(
    scores: Res<Scores>,
    p_hp: Res<PlayerHP>,
    mut query: Query<(&mut Text, &ScoreText)>,
) {
    if scores.is_changed() || p_hp.is_changed() {
        for (mut text, score_text) in query.iter_mut() {
            match score_text.0 {
                Player::One => {
                    let hearts = "❤️".repeat(p_hp.player1 as usize);
                    let skulls = "☠️".repeat(if p_hp.player1 == 0 { 1 } else { 0 });
                    text.sections[0].value =
                        format!("P1 (Arrows) : {}  {}{}", scores.player1, hearts, skulls);
                }
                Player::Two => {
                    let hearts = "❤️".repeat(p_hp.player2 as usize);
                    let skulls = "☠️".repeat(if p_hp.player2 == 0 { 1 } else { 0 });
                    text.sections[0].value =
                        format!("P2 (WASD) : {}  {}{}", scores.player2, hearts, skulls);
                }
            }
        }
    }
}

pub fn show_pause_menu(mut q: Query<&mut Style, With<PauseMenuUI>>) {
    for mut style in q.iter_mut() {
        style.display = Display::Flex;
    }
}
pub fn hide_pause_menu(mut q: Query<&mut Style, With<PauseMenuUI>>) {
    for mut style in q.iter_mut() {
        style.display = Display::None;
    }
}
pub fn show_settings_menu(mut q: Query<&mut Style, With<SettingsMenuUI>>) {
    for mut style in q.iter_mut() {
        style.display = Display::Flex;
    }
}
pub fn hide_settings_menu(mut q: Query<&mut Style, With<SettingsMenuUI>>) {
    for mut style in q.iter_mut() {
        style.display = Display::None;
    }
}

pub fn toggle_pause(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        match *state.get() {
            GameState::Playing => next_state.set(GameState::Paused),
            GameState::Paused | GameState::Settings => next_state.set(GameState::Playing),
        }
    }
}

pub fn ui_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &UIAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut settings: ResMut<GameSettings>,
    mut hp_text: Query<&mut Text, With<HPText>>,
    mut restart_writer: EventWriter<RestartGameEvent>,
) {
    for (interaction, mut color, action) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *color = Color::srgb(0.35, 0.35, 0.35).into();
                match action {
                    UIAction::Resume => next_state.set(GameState::Playing),
                    UIAction::Settings => next_state.set(GameState::Settings),
                    UIAction::Back => next_state.set(GameState::Paused),
                    UIAction::IncreaseHP => {
                        if settings.initial_hp < 10 {
                            settings.initial_hp += 1;
                        }
                    }
                    UIAction::DecreaseHP => {
                        if settings.initial_hp > 1 {
                            settings.initial_hp -= 1;
                        }
                    }
                    UIAction::Restart => {
                        restart_writer.send(RestartGameEvent);
                        next_state.set(GameState::Playing);
                    }
                }
            }
            Interaction::Hovered => {
                *color = Color::srgb(0.25, 0.25, 0.25).into();
            }
            Interaction::None => {
                *color = Color::srgb(0.15, 0.15, 0.15).into();
            }
        }
    }

    if settings.is_changed() {
        for mut text in hp_text.iter_mut() {
            text.sections[0].value = settings.initial_hp.to_string();
        }
    }
}
