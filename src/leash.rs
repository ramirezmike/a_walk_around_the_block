use bevy::prelude::*;
use std::f32::consts::FRAC_PI_2;
use crate::{AppState};
use bevy_mod_raycast::{
    ray_intersection_over_mesh, Backfaces, DefaultPluginState, DefaultRaycastingPlugin, Ray3d,
    RayCastMesh, RayCastMethod, RayCastSource, RaycastSystem,
};

pub struct LeashPlugin;
impl Plugin for LeashPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
                SystemSet::on_enter(AppState::InGame)
                    .with_system(setup)
            )
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .with_system(update_anchors)
            );
    }
}

// Marker struct for the path origin, shown by a cyan sphere
#[derive(Component)]
pub struct PathOrigin;
// Marker struct for the path pointer, shown by a cyan box
#[derive(Component)]
pub struct PathPointer;
// Marker struct for obstacles
#[derive(Component)]
struct PathObstacle;
// Marker struct for the intersection point
#[derive(Component)]
struct PathObstaclePoint;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube::default())),
            material: materials.add(Color::GREEN.into()),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        });

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere::default())),
            material: materials.add(StandardMaterial {
                unlit: true,
                base_color: Color::RED,
                ..Default::default()
            }),
            transform: Transform::from_scale(Vec3::splat(0.1)),
            visibility: Visibility {
                is_visible: false,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(PathObstaclePoint);
}

fn update_anchors(
    mut from: Query<
        &mut Transform,
        (
            With<PathOrigin>,
            Without<PathObstacle>,
            Without<PathObstaclePoint>,
        ),
    >,
    mut pointer: Query<
        &mut Transform,
        (
            With<PathPointer>,
            Without<PathOrigin>,
            Without<PathObstacle>,
        ),
    >,
    obstacles: Query<(&Handle<Mesh>, &Transform), With<PathObstacle>>,
    meshes: Res<Assets<Mesh>>,
    mut intersection_point: Query<
        (&mut Transform, &mut Visibility),
        (
            With<PathObstaclePoint>,
            Without<PathObstacle>,
            Without<PathOrigin>,
            Without<PathPointer>,
        ),
    >,
) {
    if let Ok(mut origin_transform) = from.get_single_mut() {
        let mut pointer = pointer.single_mut();
        let from = origin_transform.translation;
        let to = Vec3::new(0.0, 0.0, 0.0);
        let ray_direction = (to - from).normalize();

        // Rotate the direction indicator
        if Vec3::Z.angle_between(ray_direction) > FRAC_PI_2 {
            origin_transform.rotation =
                Quat::from_rotation_y(Vec3::X.angle_between(ray_direction));
        } else {
            origin_transform.rotation =
                Quat::from_rotation_y(-Vec3::X.angle_between(ray_direction));
        }

        let ray = Ray3d::new(from, ray_direction);
        if let Ok((mut intersection_transform, mut visible)) =
            intersection_point.get_single_mut()
        {
            // Set everything as OK in case there are no obstacle in path
            visible.is_visible = false;

            let mut closest_hit = (to - from).length();

            // Check for an obstacle on path
            for (mesh_handle, transform) in obstacles.iter() {
                if let Some(mesh) = meshes.get(mesh_handle) {
                    let mesh_to_world = transform.compute_matrix();

                    // Check for intersection with this obstacle
                    if let Some(intersection) = ray_intersection_over_mesh(
                        mesh,
                        &mesh_to_world,
                        &ray,
                        Backfaces::Cull,
                    ) {
                        // There was an intersection, check if it is before the cursor
                        // on the ray
                        let hit_distance = intersection.distance();
                        let cursor_distance = from.distance(to);
                        if hit_distance < cursor_distance && hit_distance < closest_hit {
                            intersection_transform.translation = intersection.position();
                            visible.is_visible = true;
                            closest_hit = hit_distance;
                        }
                    }
                }
            }

            pointer.scale = Vec3::new(closest_hit / 2.0, 0.05, 0.05);
            pointer.translation = Vec3::new(closest_hit / 2.0, 0.0, 0.0);
        }
    }
}
