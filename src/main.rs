#![windows_subsystem = "windows"]

use bevy::prelude::*;

mod audio;
mod asset_loading;
mod assets;
mod bot;
mod collision;
mod component_adder;
mod direction;
mod follow_text;
mod mesh;
mod game_controller;
mod game_camera;
mod game_state;
mod ingame;
mod ingame_ui;
mod leash;
mod menus;
mod player;
mod pickup;
mod title_screen;
mod target;
mod ui;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(audio::GameAudioPlugin)
        .add_plugin(assets::AssetsPlugin)
        .add_plugin(asset_loading::AssetLoadingPlugin)
        .add_plugin(bot::BotPlugin)
        .add_plugin(component_adder::ComponentAdderPlugin)
        .add_plugin(game_state::GameStatePlugin)
        .add_plugin(ingame_ui::InGameUIPlugin)
        .add_plugin(ingame::InGamePlugin)
        .add_plugin(game_controller::GameControllerPlugin)
        .add_plugin(mesh::MeshPlugin)
        .add_plugin(title_screen::TitlePlugin)
        .add_plugin(leash::LeashPlugin)
        .add_plugin(player::PlayerPlugin)
        .add_plugin(follow_text::FollowTextPlugin)
        .add_plugin(ui::text_size::TextSizePlugin)
        .add_plugin(menus::options::OptionsMenuPlugin)
        .add_plugin(pickup::PickupPlugin)
        .add_plugin(target::TargetPlugin)
        .add_state(AppState::Initial)
        .add_system_set(SystemSet::on_enter(AppState::Initial).with_system(bootstrap))
        .run();
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    Initial,
    Pause,
    Debug,
    Options,
    InGame,
    TitleScreen,
    Loading,
}

pub fn cleanup<T: Component>(mut commands: Commands, entities: Query<Entity, With<T>>) {
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Component)]
pub struct CleanupMarker;

fn bootstrap(
    mut assets_handler: asset_loading::AssetsHandler,
    mut game_assets: ResMut<assets::GameAssets>,
) {
    assets_handler.load(AppState::TitleScreen, &mut game_assets);
}
