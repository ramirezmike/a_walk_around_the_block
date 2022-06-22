use bevy::prelude::*;
use crate::AppState;

pub struct TargetPlugin;

impl Plugin for TargetPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(AppState::InGame)
                .with_system(update_targets)
        );
    }
}

#[derive(Component)]
pub struct Target;

fn update_targets() {
}
