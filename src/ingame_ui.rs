use crate::{
    assets::GameAssets, cleanup, game_state, menus, AppState, ui::text_size, player, bot
};
use bevy::prelude::*;
use bevy::ui::UiColor;

const BUTTON_SIZE: f32 = 60.0;
pub struct InGameUIPlugin;
impl Plugin for InGameUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(AppState::InGame).with_system(setup))
            .add_event::<ButtonPressedEvent>() 
            .add_event::<ButtonHoldEvent>() 
            .add_system_set(SystemSet::on_enter(AppState::ScoreDisplay)
                            .with_system(cleanup::<CleanupMarker>))
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .with_system(update_ui)
                    //.with_system(detect_round_over),
            );
    }
}

#[derive(Component)]
struct CleanupMarker;

//  fn detect_round_over(
//      game_state: Res<game_state::GameState>,
//      mut app_state: ResMut<State<AppState>>,
//  ) {
//      if !game_state.current_level_over {
//          app_state.push(AppState::ScoreDisplay).unwrap();
//      }
//  }

#[derive(Component)]
struct LeashButton {
    pub button_type: LeashButtonType
}

#[derive(PartialEq)]
pub enum LeashButtonType {
    Green,
    Red,
    Yellow,
    Blue
}

pub struct ButtonPressedEvent {
    pub button_type: LeashButtonType
}
pub struct ButtonHoldEvent {
    pub button_type: LeashButtonType
}

fn update_ui(
    game_state: Res<game_state::GameState>,
    mut score_indicators: Query<&mut Text, (With<ScoreIndicator>, Without<TimeIndicator>)>,
    mut time_indicators: Query<&mut Text, (With<TimeIndicator>, Without<ScoreIndicator>)>,
    players: Query<&player::Player, Without<bot::Bot>>,
    mut leash_buttons: Query<(&LeashButton, &mut UiColor, &mut Style)>,
    mut button_pressed_event_reader: EventReader<ButtonPressedEvent>,
    mut button_hold_event_reader: EventReader<ButtonHoldEvent>,
) {
    for mut text in score_indicators.iter_mut() {
        text.sections[0].value = game_state.score.to_string();
    }

    for mut text in time_indicators.iter_mut() {
        text.sections[0].value = format!("{:0>2}:{:0>2}", (game_state.current_time / 60.0) as usize, 
                                                  (game_state.current_time % 60.0) as usize);
    }

    for (leash_button, mut color, mut style) in leash_buttons.iter_mut() {
        style.size = Size::new(Val::Percent(BUTTON_SIZE), Val::Auto);
        if let Ok(player) = players.get_single() {
            match leash_button.button_type {
                LeashButtonType::Green => {
                    if player.south_pet.is_some() {
                        color.0 = Color::WHITE;
                    }
                },
                LeashButtonType::Red => {
                    if player.east_pet.is_some() {
                        color.0 = Color::WHITE;
                    }
                },
                LeashButtonType::Yellow => {
                    if player.north_pet.is_some() {
                        color.0 = Color::WHITE;
                    }
                },
                LeashButtonType::Blue => {
                    if player.west_pet.is_some() {
                        color.0 = Color::WHITE;
                    }
                },
            }
        }
    }

    for button_pressed in button_hold_event_reader.iter() {
        for (leash_button, mut color, mut style) in leash_buttons.iter_mut() {
            if button_pressed.button_type == leash_button.button_type {
                style.size = Size::new(Val::Percent(BUTTON_SIZE * 1.15), Val::Auto);
            }
        }
    }
    for button_pressed in button_pressed_event_reader.iter() {
        for (leash_button, mut color, mut style) in leash_buttons.iter_mut() {
            if button_pressed.button_type == leash_button.button_type {
                style.size = Size::new(Val::Percent(BUTTON_SIZE * 1.3), Val::Auto);
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    mut game_state: ResMut<game_state::GameState>,
    text_scaler: text_size::TextScaler,
) {
    commands
        .spawn_bundle(UiCameraBundle::default())
        .insert(CleanupMarker);

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(98.0)),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::FlexStart,
                flex_direction: FlexDirection::ColumnReverse,
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .insert(CleanupMarker)
        .with_children(|parent| {
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(15.0)),
                        position_type: PositionType::Relative,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::FlexEnd,
                        flex_direction: FlexDirection::Row,
                        ..Default::default()
                    },
                    color: Color::NONE.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    add_title(
                        parent,
                        game_assets.font.clone(),
                        text_scaler.scale(menus::DEFAULT_FONT_SIZE * 1.2),
                        "Score: ",
                        Vec::<CleanupMarker>::new(), // just an empty vec since can't do <impl Trait>
                    );
                    add_title(
                        parent,
                        game_assets.font.clone(),
                        text_scaler.scale(menus::DEFAULT_FONT_SIZE * 1.2),
                        "0",
                        vec!(ScoreIndicator), // just an empty vec since can't do <impl Trait>
                    );
                });
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(10.0)),
                        position_type: PositionType::Relative,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::FlexEnd,
                        flex_direction: FlexDirection::Row,
                        margin: Rect {
                            top: Val::Percent(-10.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    color: Color::NONE.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    add_title(
                        parent,
                        game_assets.font.clone(),
                        text_scaler.scale(menus::DEFAULT_FONT_SIZE * 0.6),
                        "Time: ",
                        Vec::<CleanupMarker>::new(), // just an empty vec since can't do <impl Trait>
                    );
                    add_title(
                        parent,
                        game_assets.font.clone(),
                        text_scaler.scale(menus::DEFAULT_FONT_SIZE * 0.6),
                        "00:00",
                        vec!(TimeIndicator), // just an empty vec since can't do <impl Trait>
                    );
                });

            let scale = text_scaler.scale(menus::BUTTON_LABEL_FONT_SIZE);
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(30.0)),
                        position_type: PositionType::Relative,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::FlexEnd,
                        flex_direction: FlexDirection::ColumnReverse,
                        margin: Rect {
                            top: Val::Percent(25.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    color: Color::NONE.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent
                        .spawn_bundle(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(25.0), Val::Percent(100.0)),
                                position_type: PositionType::Relative,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::FlexEnd,
                                flex_direction: FlexDirection::ColumnReverse,
                                margin: Rect {
                                    top: Val::Percent(2.0),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            color: Color::NONE.into(),
                            ..Default::default()
                        })
                        .with_children(|parent| {
                           parent 
                                .spawn_bundle(NodeBundle {
                                    style: Style {
                                        size: Size::new(Val::Percent(50.0), Val::Percent(33.0)),
                                        position_type: PositionType::Relative,
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::FlexEnd,
                                        margin: Rect {
                                            left: Val::Auto,
                                            right: Val::Auto,
                                            ..Default::default()
                                        },
                                        flex_direction: FlexDirection::Row,
                                        ..Default::default()
                                    },
                                    color: Color::NONE.into(),
                                    ..Default::default()
                                })
                                .with_children(|parent| {
                                    parent.spawn_bundle(ImageBundle {
                                        style: Style {
                                            size: Size::new(Val::Percent(BUTTON_SIZE), Val::Auto),
                                            ..Default::default()
                                        },
                                        color: bevy::ui::UiColor(Color::DARK_GRAY),
                                        image: game_assets.yellow_button.image.clone().into(),
                                        ..Default::default()
                                    })
                                    .insert(LeashButton {
                                        button_type: LeashButtonType::Yellow
                                    });
                                });
                           parent 
                                .spawn_bundle(NodeBundle {
                                    style: Style {
                                        size: Size::new(Val::Percent(100.0), Val::Percent(33.0)),
                                        position_type: PositionType::Relative,
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::FlexEnd,
                                        flex_direction: FlexDirection::Row,
                                        ..Default::default()
                                    },
                                    color: Color::NONE.into(),
                                    ..Default::default()
                                })
                                .with_children(|parent| {
                                   parent 
                                        .spawn_bundle(NodeBundle {
                                            style: Style {
                                                size: Size::new(Val::Percent(50.0), Val::Percent(100.0)),
                                                position_type: PositionType::Relative,
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::FlexEnd,
                                                flex_direction: FlexDirection::Row,
                                                ..Default::default()
                                            },
                                            color: Color::NONE.into(),
                                            ..Default::default()
                                        })
                                        .with_children(|parent| {
                                            parent.spawn_bundle(ImageBundle {
                                                style: Style {
                                                    size: Size::new(Val::Percent(BUTTON_SIZE), Val::Auto),
                                                    ..Default::default()
                                                },
                                                color: bevy::ui::UiColor(Color::DARK_GRAY),
                                                image: game_assets.blue_button.image.clone().into(),
                                                ..Default::default()
                                            })
                                            .insert(LeashButton {
                                                button_type: LeashButtonType::Blue
                                            });
                                        });
                                   parent 
                                        .spawn_bundle(NodeBundle {
                                            style: Style {
                                                size: Size::new(Val::Percent(50.0), Val::Percent(100.0)),
                                                position_type: PositionType::Relative,
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::FlexEnd,
                                                flex_direction: FlexDirection::Row,
                                                ..Default::default()
                                            },
                                            color: Color::NONE.into(),
                                            ..Default::default()
                                        })
                                        .with_children(|parent| {
                                            parent.spawn_bundle(ImageBundle {
                                                style: Style {
                                                    size: Size::new(Val::Percent(BUTTON_SIZE), Val::Auto),
                                                    ..Default::default()
                                                },
                                                color: bevy::ui::UiColor(Color::DARK_GRAY),
                                                image: game_assets.red_button.image.clone().into(),
                                                ..Default::default()
                                            })
                                            .insert(LeashButton {
                                                button_type: LeashButtonType::Red
                                            });
                                        });
                                });
                           parent 
                                .spawn_bundle(NodeBundle {
                                    style: Style {
                                        size: Size::new(Val::Percent(50.0), Val::Percent(33.0)),
                                        position_type: PositionType::Relative,
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::FlexEnd,
                                        flex_direction: FlexDirection::Row,
                                        margin: Rect {
                                            left: Val::Auto,
                                            right: Val::Auto,
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    color: Color::NONE.into(),
                                    ..Default::default()
                                })
                                .with_children(|parent| {
                                    parent.spawn_bundle(ImageBundle {
                                        style: Style {
                                            size: Size::new(Val::Percent(BUTTON_SIZE), Val::Auto),
                                            ..Default::default()
                                        },
                                        color: bevy::ui::UiColor(Color::DARK_GRAY),
                                        image: game_assets.green_button.image.clone().into(),
                                        ..Default::default()
                                    })
                                    .insert(LeashButton {
                                        button_type: LeashButtonType::Green
                                    });
                                });
                        });
                });
        });
}

#[derive(Component)]
struct ScoreIndicator;

#[derive(Component)]
struct TimeIndicator;

pub fn add_title(
    builder: &mut ChildBuilder<'_, '_, '_>,
    font: Handle<Font>,
    font_size: f32,
    title: &str,
    mut components: Vec<impl Component>,
) {
    let mut text_bundle = builder.spawn_bundle(TextBundle {
        style: Style {
            position_type: PositionType::Relative,
            margin: Rect {
//              left: Val::Percent(2.0),
//              right: Val::Auto,
                ..Default::default()
            },
            align_items: AlignItems::FlexEnd,
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        text: Text::with_section(
            title.to_string(),
            TextStyle {
                font,
                font_size,
                color: Color::WHITE,
            },
            TextAlignment {
                horizontal: HorizontalAlign::Center,
                ..Default::default()
            },
        ),
        ..Default::default()
    });

    components.drain(..).for_each(|c| {
        text_bundle.insert(c);
    });
}


