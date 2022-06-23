use crate::asset_loading;
use bevy::gltf::Gltf;
use bevy::prelude::*;
use bevy_kira_audio::AudioSource;

pub struct AssetsPlugin;
impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameAssets::default());
    }
}

#[derive(Default)]
pub struct GameAssets {
    pub font: Handle<Font>,
    pub chicken: Handle<Gltf>,
    pub chunk: Handle<Gltf>,

    pub pickup: Handle<AudioSource>,
    pub powerup: Handle<AudioSource>,
    pub attack: Handle<AudioSource>,
}

#[derive(Default)]
pub struct GameMesh {
    pub mesh: Handle<Mesh>,
    pub texture: asset_loading::GameTexture,
}
