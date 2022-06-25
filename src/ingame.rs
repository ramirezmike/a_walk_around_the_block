use crate::{
    asset_loading, assets::GameAssets, bot, cleanup, collision, component_adder, game_camera,
    game_state, leash, player, target, AppState, CleanupMarker, audio::GameAudio
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
    assets_handler.add_glb(&mut game_assets.dog, "models/dog.glb");
    assets_handler.add_glb(&mut game_assets.person, "models/person.glb");
    assets_handler.add_glb(&mut game_assets.person_02, "models/person_02.glb");
    assets_handler.add_glb(&mut game_assets.person_03, "models/person_03.glb");
    assets_handler.add_glb(&mut game_assets.person_04, "models/person_04.glb");
    assets_handler.add_glb(&mut game_assets.chickendog, "models/chickendog.glb");
    assets_handler.add_glb(&mut game_assets.chip, "models/chip.glb");
    assets_handler.add_glb(&mut game_assets.worm, "models/worm.glb");
    assets_handler.add_glb(&mut game_assets.chunk, "models/chunk.glb");
    assets_handler.add_glb(&mut game_assets.poop, "models/poop.glb");

    assets_handler.add_audio(&mut game_assets.pickup, "audio/pickup.wav");
    assets_handler.add_audio(&mut game_assets.powerup, "audio/powerup.wav");
    assets_handler.add_audio(&mut game_assets.attack, "audio/attack.wav");
    assets_handler.add_audio(&mut game_assets.game_music, "audio/happy_panda.ogg");

    assets_handler.add_font(&mut game_assets.font, "fonts/monogram.ttf");

    assets_handler.add_material(&mut game_assets.green_button, "textures/green_button.png", true);
    assets_handler.add_material(&mut game_assets.red_button, "textures/red_button.png", true);
    assets_handler.add_material(&mut game_assets.blue_button, "textures/blue_button.png", true);
    assets_handler.add_material(&mut game_assets.yellow_button, "textures/yellow_button.png", true);
}

fn setup(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    asset_server: Res<AssetServer>,
    assets_gltf: Res<Assets<Gltf>>,
    game_state: Res<game_state::GameState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut grid_materials: ResMut<Assets<InfiniteGridMaterial>>,
    mut component_adder: ResMut<component_adder::ComponentAdder>,
    mut new_chunk_event_writer: EventWriter<game_state::NewChunkEvent>,
    mut audio: GameAudio,
) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.50,
    });

    let model = game_assets.get_random_player_model();
    if let Some(gltf) = assets_gltf.get(&model) {
        let mut player = commands
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
                        parent.spawn_scene(gltf.scenes[0].clone());
                    });
                })
                .insert(leash::Anchor {
                    parent: None,
                    leash: None,
                })
                .insert_bundle(player::PlayerBundle::new(None))
                .insert(CleanupMarker);
    }

    component_adder.reset();
    new_chunk_event_writer.send(game_state::NewChunkEvent);

    if game_state.music_on {
        audio.play_bgm(&game_assets.game_music);
    } else {
        audio.stop_bgm();
    }
}
