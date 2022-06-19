#![windows_subsystem = "windows"]

use bevy::prelude::*;

mod ingame;
mod game_camera;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ingame::InGamePlugin)
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
