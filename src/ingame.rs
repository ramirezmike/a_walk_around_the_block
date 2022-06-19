use crate::{cleanup, game_camera, AppState, player, leash};
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridMaterial, InfiniteGridPlugin};
use bevy::prelude::*;

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
                .with_system(game_camera::follow_player.after("move_player"))
        );
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut grid_materials: ResMut<Assets<InfiniteGridMaterial>>,
) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.50,
    });

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    })
    .insert(leash::PathOrigin)
    .with_children(|from| {
        // Spawn a visual indicator for the path direction
        from.spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::default())),
            material: materials.add(StandardMaterial {
                unlit: true,
                base_color: Color::RED,
                ..Default::default()
            }),
            transform: Transform::from_scale(Vec3::ZERO),
            ..Default::default()
        })
        .insert(leash::PathPointer);
    })
    .insert_bundle(player::PlayerBundle::default());

    commands.spawn_bundle(InfiniteGridBundle::new(
        grid_materials.add(InfiniteGridMaterial::default()),
    ));
}
