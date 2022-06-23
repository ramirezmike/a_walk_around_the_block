use crate::{
    asset_loading, assets::GameAssets, bot, cleanup, collision, component_adder, game_camera,
    game_state, leash, player, target, AppState, CleanupMarker,
};
use bevy::gltf::Gltf;
use bevy::prelude::*;
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridMaterial, InfiniteGridPlugin};

pub struct InGamePlugin;
impl Plugin for InGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::InGame)
                .with_system(game_camera::spawn_camera)
                .with_system(setup),
        )
        .add_plugin(InfiniteGridPlugin)
        .add_system_set(
            SystemSet::on_update(AppState::InGame)
                .with_system(game_camera::follow_player.after("move_player")),
        );
    }
}

pub fn load(
    assets_handler: &mut asset_loading::AssetsHandler,
    game_assets: &mut ResMut<GameAssets>,
) {
    assets_handler.add_glb(&mut game_assets.chicken, "models/chicken.glb");
    assets_handler.add_glb(&mut game_assets.chunk, "models/chunk.glb");
}

fn setup(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    asset_server: Res<AssetServer>,
    assets_gltf: Res<Assets<Gltf>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut grid_materials: ResMut<Assets<InfiniteGridMaterial>>,
    mut component_adder: ResMut<component_adder::ComponentAdder>,
    mut new_chunk_event_writer: EventWriter<game_state::NewChunkEvent>,
) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.50,
    });

    if let Some(gltf) = assets_gltf.get(&game_assets.chunk) {
        commands
            .spawn_bundle((
                Transform::from_xyz(0.0, -0.5, 0.0),
                GlobalTransform::identity(),
            ))
            .insert(game_state::Chunk {
                position: Vec2::new(0.0, 0.0),
            })
            .insert(CleanupMarker)
            .with_children(|parent| {
                parent.spawn_scene(gltf.scenes[0].clone());
            });
    }

    if let Some(gltf) = assets_gltf.get(&game_assets.chicken) {
        let mut player = commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube::default())),
            material: materials.add(Color::GREEN.into()),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        });

        let player_id = player.id();
        player
            .insert(leash::Anchor {
                parent: None,
                leash: None,
            })
            .insert(CleanupMarker);

        let leash = commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Box::default())),
                material: materials.add(StandardMaterial {
                    unlit: true,
                    base_color: Color::RED,
                    ..Default::default()
                }),
                transform: Transform::from_scale(Vec3::ZERO),
                ..Default::default()
            })
            .insert(leash::Leash)
            .insert(CleanupMarker)
            .id();

        let chicken_id = commands
            .spawn_bundle((
                Transform::from_xyz(0.0, 0.0, 0.0),
                GlobalTransform::identity(),
            ))
            .with_children(|parent| {
                parent
                    .spawn_bundle((
                        Transform::from_rotation(Quat::from_rotation_y(
                            std::f32::consts::FRAC_PI_2,
                        )),
                        GlobalTransform::identity(),
                    ))
                    .with_children(|parent| {
                        parent.spawn_scene(asset_server.load("models/chicken.glb#Scene0"));
                    });
            })
            .insert_bundle(bot::BotBundle::new())
            .insert(CleanupMarker)
            .insert(bot::Pet {
                pet_type: bot::PetType::Chicken,
            })
            .insert(leash::Anchor {
                parent: Some(player_id),
                leash: Some(leash),
            })
            .id();

        commands
            .entity(player_id)
            .insert_bundle(player::PlayerBundle::new(Some(chicken_id)));

        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube::default())),
                material: materials.add(Color::BLUE.into()),
                transform: Transform::from_xyz(6.0, 0.0, 6.0),
                ..Default::default()
            })
            .insert(CleanupMarker)
            .insert(target::Target::new());
    }

    component_adder.reset();

    new_chunk_event_writer.send(game_state::NewChunkEvent);
}
