#![windows_subsystem = "windows"]

use bevy::prelude::*;

mod direction;
mod game_camera;
mod ingame;
mod leash;
mod player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ingame::InGamePlugin)
        .add_plugin(leash::LeashPlugin)
        .add_plugin(player::PlayerPlugin)
        .add_state(AppState::InGame)
        .run();
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    Initial,
    Pause,
    Debug,
    InGame,
    Loading,
}

pub fn cleanup<T: Component>(mut commands: Commands, entities: Query<Entity, With<T>>) {
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
