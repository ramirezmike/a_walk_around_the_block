use crate::{cleanup, game_camera, leash, player, AppState, collision, component_adder};
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

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut grid_materials: ResMut<Assets<InfiniteGridMaterial>>,
    mut component_adder: ResMut<component_adder::ComponentAdder>,
) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.50,
    });

    let mut player = commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
    let player_id = player.id();
    player
        .insert(leash::Anchor {
            parent: None,
            leash: None,
        })
        .insert_bundle(player::PlayerBundle::default());

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
        .id();

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube::default())),
            material: materials.add(Color::GREEN.into()),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        })
        .push_children(&[leash])
        .insert(leash::Anchor {
            parent: Some(player_id),
            leash: Some(leash),
        });

    let size = 2.0;
    let spacing = 6.0;

    // Spawn obstacles
    for x in -2..=2 {
        for z in -2..=2 {
            if x as f32 * spacing == 0.0 && z as f32 * spacing == 0.0 {
                continue;
            }
            commands
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube::new(size))),
                    material: materials.add(Color::BLACK.into()),
                    transform: Transform::from_xyz(x as f32 * spacing, 0.0, z as f32 * spacing),
                    ..Default::default()
                })
                .insert(Name::new(format!("collidable_{}_{}", x, z)))
                .insert(leash::PathObstacle);
        }
    }

    commands.spawn_bundle(InfiniteGridBundle::new(
        grid_materials.add(InfiniteGridMaterial::default()),
    ));

    component_adder.reset();
}
