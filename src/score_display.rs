use crate::{
    assets::GameAssets, cleanup, game_camera, game_state, menus, player, ui::text_size,
    AppState, CleanupMarker
};
use bevy::prelude::*;

pub struct ScoreDisplayPlugin;
impl Plugin for ScoreDisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(AppState::ScoreDisplay).with_system(setup))
            .insert_resource(ScoreState::default())
            .add_system_set(
                SystemSet::on_update(AppState::ScoreDisplay)
                    .with_system(display_scores)
            )
            .add_system_set(
                SystemSet::on_exit(AppState::ScoreDisplay).with_system(destroy_everything),
            );
    }
}

fn destroy_everything(mut commands: Commands, entities: Query<Entity>) {
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

struct ScoreState {
    cooldown: f32,
    first_render: bool,
}

impl Default for ScoreState {
    fn default() -> Self {
        ScoreState {
            cooldown: 0.0,
            first_render: true,
        }
    }
}

fn setup(mut score_state: ResMut<ScoreState>, mut game_state: ResMut<game_state::GameState>) {
    *score_state = ScoreState::default();
}

fn display_scores(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    game_state: Res<game_state::GameState>,
    mut app_state: ResMut<State<AppState>>,
    mut score_state: ResMut<ScoreState>,
    cleanups: Query<Entity, With<CleanupMarker>>,
    time: Res<Time>,
    text_scaler: text_size::TextScaler,
) {
    score_state.cooldown -= time.delta_seconds();
    score_state.cooldown = score_state.cooldown.clamp(-10.0, 3.0);

    if !score_state.first_render {
        if score_state.cooldown <= 0.0 {
            app_state.set(AppState::TitleScreen).unwrap();
        } else {
            return;
        }
    }
    score_state.first_render = false;
    score_state.cooldown = 6.0;

    commands
        .spawn_bundle(UiCameraBundle::default())
        .insert(CleanupMarker);

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(35.0)),
                margin: Rect::all(Val::Auto),
                position_type: PositionType::Relative,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexEnd,
                flex_direction: FlexDirection::ColumnReverse,
                ..Default::default()
            },
            color: Color::rgba(0.3, 0.3, 0.3, 0.4).into(),
            ..Default::default()
        })
        .insert(CleanupMarker)
        .with_children(|parent| {
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(20.0)),
                        position_type: PositionType::Relative,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::FlexEnd,
                        ..Default::default()
                    },
                    color: Color::NONE.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    menus::options::add_title(
                        parent,
                        game_assets.font.clone(),
                        text_scaler.scale(menus::DEFAULT_FONT_SIZE * 1.2),
                        if !game_state.lost_pet {
                            "You Won!"
                        } else {
                            "Game Over"
                        },
                        Vec::<CleanupMarker>::new(), // just an empty vec since can't do <impl Trait>
                    );
                });

            let score_text = format!("Score: {}", game_state.score);
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(20.0)),
                        position_type: PositionType::Relative,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::FlexEnd,
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
                    menus::options::add_title(
                        parent,
                        game_assets.font.clone(),
                        text_scaler.scale(menus::DEFAULT_FONT_SIZE * 1.1),
                        if !game_state.lost_pet {
                            score_text.as_str()
                        } else {
                            "omg you lost a pet!"
                        },
                        Vec::<CleanupMarker>::new(), // just an empty vec since can't do <impl Trait>
                    );
                });
        });
}
