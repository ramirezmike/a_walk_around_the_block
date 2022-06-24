use crate::{game_camera::PanOrbitCamera, CleanupMarker, ui::text_size, menus, assets::GameAssets};
use bevy::prelude::*;

pub struct FollowTextPlugin;
impl Plugin for FollowTextPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_text_position)
            .add_system(create_follow_text)
            .add_event::<FollowTextEvent>();
    }
}

#[derive(Component)]
pub struct FollowText {
    pub following: FollowThing,
    pub offset: f32,
    pub time_to_live: f32,
}

pub struct FollowTextEvent {
    pub follow: FollowThing,
    pub text: String,
    pub color: Color,
    pub time_to_live: f32,
}

#[derive(Copy, Clone)]
pub enum FollowThing {
    Entity(Entity),
    Spot(Vec3)
}

fn update_text_position(
    mut commands: Commands,
    windows: Res<Windows>,
    mut text_query: Query<(Entity, &mut Style, &CalculatedSize, &mut FollowText)>,
    mesh_query: Query<&Transform>,
    camera_query: Query<(&Camera, &GlobalTransform), With<PanOrbitCamera>>,
    images: Res<Assets<Image>>,
    time: Res<Time>,
) {
    for (entity, mut style, calculated, mut follow) in text_query.iter_mut() {
        let translation = match follow.following {
                            FollowThing::Entity(e) => {
                                if let Ok(mesh_position) = mesh_query.get(e) {
                                    Vec3::new(
                                        mesh_position.translation.x,
                                        mesh_position.translation.y + 1.0 + follow.offset,
                                        mesh_position.translation.z,
                                    )
                                } else {
                                    Vec3::ZERO
                                }
                            },
                            FollowThing::Spot(v) => v
                          };
        for (camera, camera_transform) in camera_query.iter() {
            follow.offset += time.delta_seconds() * 2.0;
            match camera.world_to_screen(&windows, &images, camera_transform, translation) {
                Some(coords) => {
                    style.position.left = Val::Px(coords.x - calculated.size.width / 2.0);
                    style.position.bottom = Val::Px((coords.y) - calculated.size.height / 2.0);
                }
                None => {
                    // A hack to hide the text when the it's behind the camera
                    style.position.bottom = Val::Px(-1000.0);
                }
            }
        }

        if follow.offset > follow.time_to_live {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn create_follow_text(
    mut commands: Commands,
    mut game_assets: ResMut<GameAssets>,
    text_scaler: text_size::TextScaler,
    mut follow_text_event_reader: EventReader<FollowTextEvent>,
) {
    for event in follow_text_event_reader.iter() {
        commands
            .spawn_bundle(TextBundle {
                style: Style {
                    align_self: AlignSelf::FlexEnd,
                    position_type: PositionType::Absolute,
                    position: Rect {
                        bottom: Val::Px(5.0),
                        left: Val::Px(15.0),
                        ..Default::default()
                    },
                    size: Size {
                        width: Val::Percent(50.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                text: Text::with_section(
                    event.text.clone(),
                    TextStyle {
                        font: game_assets.font.clone(),
                        font_size: text_scaler.scale(menus::FOLLOW_FONT_SIZE),
                        color: event.color,
                    },
                    TextAlignment {
                        ..Default::default()
                    },
                ),
                ..Default::default()
            })
            .insert(CleanupMarker)
            .insert(FollowText { following: event.follow, offset: 0.0, time_to_live: event.time_to_live });
    }
}
