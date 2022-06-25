use crate::asset_loading;
use bevy::gltf::Gltf;
use bevy::prelude::*;
use bevy_kira_audio::AudioSource;
use rand::seq::SliceRandom;
use rand::thread_rng;

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
    pub person_02: Handle<Gltf>,
    pub person_03: Handle<Gltf>,
    pub person_04: Handle<Gltf>,
    pub chickendog: Handle<Gltf>,
    pub poop: Handle<Gltf>,
    pub chip: Handle<Gltf>,
    pub worm: Handle<Gltf>,
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

impl GameAssets {
    pub fn get_random_player_model(&self) -> Handle<Gltf> {
        let mut rng = thread_rng();
        let models = vec!(self.person.clone(), self.person_02.clone(), 
                          self.person_03.clone(), self.person_04.clone());
        let model = models.choose(&mut rng).unwrap_or(&self.person_03);
        model.clone()
    }
}


#[derive(Default)]
pub struct GameMesh {
    pub mesh: Handle<Mesh>,
    pub texture: asset_loading::GameTexture,
}
