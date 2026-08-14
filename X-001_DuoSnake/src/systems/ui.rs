use crate::components::*;
use crate::constants::*;
use crate::pixel_art::PixelAssets;
use bevy::prelude::*;

/// Multiplies a palette colour's brightness, used for hover / press states.
fn shade(rgba: [u8; 4], factor: f32) -> Color {
    let scale = |v: u8| (v as f32 * factor).clamp(0.0, 255.0) as u8;
    Color::srgba_u8(scale(rgba[0]), scale(rgba[1]), scale(rgba[2]), rgba[3])
}

fn spawn_button(parent: &mut ChildBuilder, label: &str, action: UIAction, accent: [u8; 4]) {
    let theme = ButtonTheme {
        normal: col(accent),
        hovered: shade(accent, 1.1),
        pressed: shade(accent, 0.82),
    };

    parent
        .spawn((
            ButtonBundle {
                style: Style {
                    min_width: Val::Px(210.0),
                    height: Val::Px(64.0),
                    margin: UiRect::all(Val::Px(10.0)),
                    padding: UiRect::horizontal(Val::Px(22.0)),
                    border: UiRect::all(Val::Px(4.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: theme.normal.into(),
                border_color: shade(accent, 0.7).into(),
                border_radius: BorderRadius::all(Val::Px(18.0)),
                ..default()
            },
            theme,
            action,
        ))
        .with_children(|button| {
            button.spawn(TextBundle::from_section(
                label,
                TextStyle {
                    font_size: 30.0,
                    color: col(COL_INK),
                    ..default()
                },
            ));
        });
}

/// Small square button, for the `-` / `+` steppers.
fn spawn_stepper(parent: &mut ChildBuilder, label: &str, action: UIAction, accent: [u8; 4]) {
    let theme = ButtonTheme {
        normal: col(accent),
        hovered: shade(accent, 1.1),
        pressed: shade(accent, 0.82),
    };

    parent
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(56.0),
                    height: Val::Px(56.0),
                    margin: UiRect::horizontal(Val::Px(14.0)),
                    border: UiRect::all(Val::Px(4.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: theme.normal.into(),
                border_color: shade(accent, 0.7).into(),
                border_radius: BorderRadius::all(Val::Px(16.0)),
                ..default()
            },
            theme,
            action,
        ))
        .with_children(|button| {
            button.spawn(TextBundle::from_section(
                label,
                TextStyle {
                    font_size: 34.0,
                    color: col(COL_INK),
                    ..default()
                },
            ));
        });
}

/// Cream panel with a thick candy border, shared by both menus.
fn panel(parent: &mut ChildBuilder, build: impl FnOnce(&mut ChildBuilder)) {
    parent
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(56.0), Val::Px(36.0)),
                border: UiRect::all(Val::Px(6.0)),
                ..default()
            },
            background_color: col(COL_BG).into(),
            border_color: col(COL_FRAME).into(),
            border_radius: BorderRadius::all(Val::Px(28.0)),
            ..default()
        })
        .with_children(build);
}

fn heading(parent: &mut ChildBuilder, text: &str) {
    parent.spawn(
        TextBundle::from_section(
            text,
            TextStyle {
                font_size: 64.0,
                color: col(COL_INK),
                ..default()
            },
        )
        .with_style(Style {
            margin: UiRect::bottom(Val::Px(18.0)),
            ..default()
        }),
    );
}

fn hint(parent: &mut ChildBuilder, text: &str) {
    parent.spawn(
        TextBundle::from_section(
            text,
            TextStyle {
                font_size: 20.0,
                color: col_a(COL_INK, 0.65),
                ..default()
            },
        )
        .with_style(Style {
            margin: UiRect::top(Val::Px(16.0)),
            ..default()
        }),
    );
}

pub fn setup_menus(mut commands: Commands) {
    // ---- pause ----
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
                background_color: col_a(COL_INK, 0.55).into(),
                z_index: ZIndex::Global(100),
                ..default()
            },
            PauseMenuUI,
        ))
        .with_children(|root| {
            panel(root, |p| {
                heading(p, "PAUSED");
                spawn_button(p, "Resume", UIAction::Resume, COL_P1);
                spawn_button(p, "Settings", UIAction::Settings, COL_P2);
                spawn_button(p, "Restart", UIAction::Restart, COL_CANDY_PINK);
                hint(p, "P1  ARROW KEYS        P2  W A S D        ESC  resume");
            });
        });

    // ---- settings ----
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
                background_color: col_a(COL_INK, 0.7).into(),
                z_index: ZIndex::Global(101),
                ..default()
            },
            SettingsMenuUI,
        ))
        .with_children(|root| {
            panel(root, |p| {
                heading(p, "SETTINGS");

                p.spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(12.0)),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|row| {
                    row.spawn(
                        TextBundle::from_section(
                            "Starting HP",
                            TextStyle {
                                font_size: 34.0,
                                color: col(COL_INK),
                                ..default()
                            },
                        )
                        // Without this the label is squeezed and wraps mid-phrase.
                        .with_style(Style {
                            flex_shrink: 0.0,
                            margin: UiRect::right(Val::Px(10.0)),
                            ..default()
                        }),
                    );
                    spawn_stepper(row, "-", UIAction::DecreaseHP, COL_CANDY_PINK);
                    row.spawn((
                        TextBundle::from_section(
                            "5",
                            TextStyle {
                                font_size: 40.0,
                                color: col(COL_INK),
                                ..default()
                            },
                        )
                        .with_text_justify(JustifyText::Center)
                        // Fixed width so 9 -> 10 does not nudge the steppers.
                        .with_style(Style {
                            width: Val::Px(64.0),
                            flex_shrink: 0.0,
                            ..default()
                        }),
                        HPText,
                    ));
                    spawn_stepper(row, "+", UIAction::IncreaseHP, COL_P1);
                });

                spawn_button(p, "Back", UIAction::Back, COL_FRAME);
                hint(p, "Starting HP applies on the next Restart");
            });
        });
}

/// One player's candy card: caption, score and a row of heart pips.
fn player_card(parent: &mut ChildBuilder, player: Player, assets: &PixelAssets) {
    let (caption, accent, accent_dark) = match player {
        Player::One => ("P1  ARROW KEYS", COL_P1, COL_P1_DARK),
        Player::Two => ("P2  W A S D", COL_P2, COL_P2_DARK),
    };

    parent
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexStart,
                    padding: UiRect::axes(Val::Px(22.0), Val::Px(10.0)),
                    border: UiRect::all(Val::Px(4.0)),
                    row_gap: Val::Px(4.0),
                    ..default()
                },
                background_color: col_a(accent, 0.28).into(),
                border_color: col(accent).into(),
                border_radius: BorderRadius::all(Val::Px(18.0)),
                ..default()
            },
            PlayerCard(player),
        ))
        .with_children(|card| {
            card.spawn((
                TextBundle::from_section(
                    caption,
                    TextStyle {
                        font_size: 18.0,
                        color: col(accent_dark),
                        ..default()
                    },
                ),
                PlayerLabel(player),
            ));

            card.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(16.0),
                    ..default()
                },
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    TextBundle::from_section(
                        "0",
                        TextStyle {
                            font_size: 38.0,
                            color: col(accent_dark),
                            ..default()
                        },
                    )
                    .with_style(Style {
                        min_width: Val::Px(52.0),
                        ..default()
                    }),
                    ScoreText(player),
                ));

                for index in 0..MAX_HP {
                    row.spawn((
                        ImageBundle {
                            image: UiImage::new(assets.heart.clone()),
                            style: Style {
                                width: Val::Px(22.0),
                                height: Val::Px(22.0),
                                margin: UiRect::right(Val::Px(1.0)),
                                ..default()
                            },
                            ..default()
                        },
                        HeartIcon { player, index },
                    ));
                }
            });
        });
}

pub fn spawn_ui(mut commands: Commands, assets: Res<PixelAssets>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(HUD_HEIGHT),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(40.0), Val::Px(8.0)),
                border: UiRect::bottom(Val::Px(5.0)),
                ..default()
            },
            background_color: col(COL_BG).into(),
            border_color: col(COL_FRAME).into(),
            ..default()
        })
        .with_children(|bar| {
            player_card(bar, Player::Two, &assets);

            bar.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            })
            .with_children(|center| {
                center.spawn(TextBundle::from_section(
                    "SNAKE  DUO",
                    TextStyle {
                        font_size: 30.0,
                        color: col(COL_INK),
                        ..default()
                    },
                ));
                center.spawn(TextBundle::from_section(
                    "ESC  pause",
                    TextStyle {
                        font_size: 17.0,
                        color: col_a(COL_INK, 0.6),
                        ..default()
                    },
                ));
            });

            player_card(bar, Player::One, &assets);
        });
}

pub fn update_hud(
    scores: Res<Scores>,
    p_hp: Res<PlayerHP>,
    settings: Res<GameSettings>,
    assets: Res<PixelAssets>,
    mut q_scores: Query<(&mut Text, &ScoreText), Without<PlayerLabel>>,
    mut q_labels: Query<(&mut Text, &PlayerLabel), Without<ScoreText>>,
    mut q_hearts: Query<(&mut UiImage, &mut Style, &HeartIcon)>,
    mut q_cards: Query<(&mut BackgroundColor, &mut BorderColor, &PlayerCard)>,
) {
    if !(scores.is_changed() || p_hp.is_changed() || settings.is_changed()) {
        return;
    }

    let hp_of = |player: Player| match player {
        Player::One => p_hp.player1,
        Player::Two => p_hp.player2,
    };
    let score_of = |player: Player| match player {
        Player::One => scores.player1,
        Player::Two => scores.player2,
    };
    let accent_of = |player: Player| match player {
        Player::One => (COL_P1, COL_P1_DARK),
        Player::Two => (COL_P2, COL_P2_DARK),
    };

    for (mut text, score_text) in q_scores.iter_mut() {
        let player = score_text.0;
        let (_, accent_dark) = accent_of(player);
        text.sections[0].value = score_of(player).to_string();
        text.sections[0].style.color = if hp_of(player) == 0 {
            col(COL_DEAD)
        } else {
            col(accent_dark)
        };
    }

    for (mut text, label) in q_labels.iter_mut() {
        let player = label.0;
        let (_, accent_dark) = accent_of(player);
        let out = hp_of(player) == 0;
        text.sections[0].value = match (player, out) {
            (_, true) => "K.O.".to_string(),
            (Player::One, false) => "P1  ARROW KEYS".to_string(),
            (Player::Two, false) => "P2  W A S D".to_string(),
        };
        text.sections[0].style.color = if out {
            col(COL_DEAD)
        } else {
            col(accent_dark)
        };
    }

    for (mut image, mut style, heart) in q_hearts.iter_mut() {
        // Only show as many pips as the round was configured with.
        style.display = if heart.index < settings.initial_hp {
            Display::Flex
        } else {
            Display::None
        };

        let wanted = if heart.index < hp_of(heart.player) {
            assets.heart.clone()
        } else {
            assets.heart_empty.clone()
        };
        if image.texture != wanted {
            image.texture = wanted;
        }
    }

    for (mut background, mut border, card) in q_cards.iter_mut() {
        let player = card.0;
        let (accent, _) = accent_of(player);
        if hp_of(player) == 0 {
            *background = col_a(COL_DEAD, 0.28).into();
            *border = col(COL_DEAD).into();
        } else {
            *background = col_a(accent, 0.28).into();
            *border = col(accent).into();
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
        (&Interaction, &mut BackgroundColor, &ButtonTheme, &UIAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut settings: ResMut<GameSettings>,
    mut hp_text: Query<&mut Text, With<HPText>>,
    mut restart_writer: EventWriter<RestartGameEvent>,
) {
    for (interaction, mut color, theme, action) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *color = theme.pressed.into();
                match action {
                    UIAction::Resume => next_state.set(GameState::Playing),
                    UIAction::Settings => next_state.set(GameState::Settings),
                    UIAction::Back => next_state.set(GameState::Paused),
                    UIAction::IncreaseHP => {
                        if settings.initial_hp < MAX_HP {
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
            Interaction::Hovered => *color = theme.hovered.into(),
            Interaction::None => *color = theme.normal.into(),
        }
    }

    if settings.is_changed() {
        for mut text in hp_text.iter_mut() {
            text.sections[0].value = settings.initial_hp.to_string();
        }
    }
}
