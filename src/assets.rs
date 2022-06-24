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
    pub dog: Handle<Gltf>,
    pub person: Handle<Gltf>,
    pub chunk: Handle<Gltf>,

    pub pickup: Handle<AudioSource>,
    pub blip: Handle<AudioSource>,
    pub powerup: Handle<AudioSource>,
    pub attack: Handle<AudioSource>,
    pub titlescreen: Handle<AudioSource>,
    pub game_music: Handle<AudioSource>,

    pub green_button: asset_loading::GameTexture,
    pub red_button: asset_loading::GameTexture,
    pub blue_button: asset_loading::GameTexture,
    pub yellow_button: asset_loading::GameTexture,
    pub title_screen_background: asset_loading::GameTexture,
    pub title_screen_logo: asset_loading::GameTexture,
}

#[derive(Default)]
pub struct GameMesh {
    pub mesh: Handle<Mesh>,
    pub texture: asset_loading::GameTexture,
}
