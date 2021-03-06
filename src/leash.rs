use crate::AppState;
use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use bevy_mod_raycast::{
    ray_intersection_over_mesh, Backfaces, DefaultPluginState, DefaultRaycastingPlugin, Ray3d,
    RayCastMesh, RayCastMethod, RayCastSource, RaycastSystem,
};
use std::cmp::Ordering;
use std::f32::consts::FRAC_PI_2;

pub struct LeashPlugin;
impl Plugin for LeashPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(AppState::InGame).with_system(setup))
            .add_event::<CreateAnchorEvent>()
            .add_event::<RemoveAnchorEvent>()
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .with_system(update_anchors.label("update_anchors"))
                    .with_system(
                        handle_remove_anchor
                            .label("remove_anchors")
                            .after("update_anchors"),
                    )
                    .with_system(
                        handle_create_anchor
                            .label("create_anchors")
                            .after("remove_anchors"),
                    )
                    .with_system(update_leashes.after("create_anchors")), //                    .with_system(print_anchors)
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
pub struct PathObstacle;

#[derive(Component)]
pub struct Leash {
    pub color: Color
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Anchor {
    pub parent: Option<Entity>,
    pub leash: Option<Entity>,
}

fn update_leashes(
    anchors: Query<(&Transform, &Anchor)>,
    mut leashes: Query<&mut Transform, (Without<Anchor>, With<Leash>)>,
) {
    for (child_transform, child_anchor) in anchors.iter() {
        if let Some(parent) = child_anchor.parent {
            if let Ok((parent_transform, parent_anchor)) = anchors.get(parent) {
                if let Some(leash) = child_anchor.leash {
                    if let Ok(mut leash) = leashes.get_mut(leash) {
                        let diff =
                            (child_transform.translation - parent_transform.translation).length();
                        let middle = child_transform
                            .translation
                            .lerp(parent_transform.translation, 0.5);
                        let direction = (parent_transform.translation
                            - child_transform.translation)
                            .normalize();

                        leash.scale = Vec3::new(diff / 2.0, 0.05, 0.05);
                        leash.translation = Vec3::new(middle.x, 1.0, middle.z);

                        // Rotate the direction indicator
                        if Vec3::Z.angle_between(direction) > FRAC_PI_2 {
                            leash.rotation =
                                Quat::from_rotation_y(Vec3::X.angle_between(direction));
                        } else {
                            leash.rotation =
                                Quat::from_rotation_y(-Vec3::X.angle_between(direction));
                        }
                    }
                }
            }
        }
    }
}

struct CreateAnchorEvent {
    parent: Entity,
    position: Vec3,
    child: Entity,
}

struct RemoveAnchorEvent {
    parent: Entity,
    new_parent: Entity,
    child: Entity,
}

fn update_anchors(
    anchors: Query<(Entity, &Transform, &Anchor), Without<Leash>>,
    obstacles: Query<
        (&Handle<Mesh>, &Transform, &Aabb, &GlobalTransform),
        (With<PathObstacle>, Without<Leash>),
    >,
    mut create_anchor_event_writer: EventWriter<CreateAnchorEvent>,
    mut remove_anchor_event_writer: EventWriter<RemoveAnchorEvent>,
    meshes: Res<Assets<Mesh>>,
) {
    //   println!("Number of obstacles: {:?}", obstacles.iter().count());
    /*
        if an obstacle is detected, throw an event to spawn a new anchor with the parent and the child
        need to make the new anchor point to the parent and make the child point to the new anchor

        after we handle that, check if any child can see their grand parent
        if so, throw an event to remove the parent
        need to set the child to point to the grand parent
    */

    for (child_entity, child_transform, child_anchor) in anchors.iter() {
        if let Some((_, parent_transform, parent_anchor)) =
            child_anchor.parent.and_then(|e| anchors.get(e).ok())
        {
            let parent_entity = child_anchor.parent.unwrap();
            let from = parent_transform.translation;
            let to = child_transform.translation;
            let ray_direction = (to - from).normalize();

            let ray = Ray3d::new(from, ray_direction);
            let mut closest_hit = (to - from).length();
            let mut new_anchor_was_created = false;

            // Check for an obstacle on path
            for (mesh_handle, transform, aabb, global_transform) in obstacles.iter() {
                if let Some(mesh) = meshes.get(mesh_handle) {
                    let mesh_to_world = global_transform.compute_matrix();

                    // Check for intersection with this obstacle
                    if let Some(intersection) =
                        ray_intersection_over_mesh(mesh, &mesh_to_world, &ray, Backfaces::Cull)
                    {
                        let hit_distance = intersection.distance();
                        let cursor_distance = from.distance(to);
                        if hit_distance < cursor_distance && hit_distance < closest_hit {
                            let mut cloned_global_transform = global_transform.clone();
                            cloned_global_transform.scale *= 1.01;
                            let matrix = cloned_global_transform.compute_matrix();

                            //                              println!("-----");
                            //                              println!("Min: {:?}", matrix.transform_point3(aabb.min().into()));
                            //                              println!("Max: {:?}", matrix.transform_point3(aabb.max().into()));
                            //                              println!("Scale: {:?}", global_transform.scale);

                            let intersection_point = intersection.position();
                            let min = matrix.transform_point3(aabb.min().into());
                            let max = matrix.transform_point3(aabb.max().into());
                            let mut points: Vec<(f32, Vec3)> = vec![
                                Vec3::new(min.x, 0.0, min.z),
                                Vec3::new(max.x, 0.0, min.z),
                                Vec3::new(min.x, 0.0, max.z),
                                Vec3::new(max.x, 0.0, max.z),
                            ]
                            .into_iter()
                            .map(|p| (p.distance(intersection_point), p))
                            .collect::<Vec<_>>();

                            //                              for i in points.iter() {
                            //                                  println!("D: {:?} P: {:?}", i.0, i.1);
                            //                              }

                            points.sort_by(|(distance_a, _), (distance_b, _)| {
                                distance_a
                                    .partial_cmp(distance_b)
                                    .unwrap_or(Ordering::Equal)
                            });
                            let anchor_point = points[0].1;
                            //                              println!("Anchor pt: {:?}", anchor_point);
                            //                              println!("Child pt: {:?}", child_transform.translation);
                            //                              println!("I: {:?}", intersection.position());

                            if anchor_point == child_transform.translation
                                || anchor_point == parent_transform.translation
                            {
                                continue;
                            }

                            create_anchor_event_writer.send(CreateAnchorEvent {
                                parent: parent_entity,
                                position: anchor_point,
                                child: child_entity,
                            });

                            //                              println!("C: {:?}", child_transform.translation);
                            //                              println!("P: {:?}", parent_transform.translation);
                            //                              println!("");

                            new_anchor_was_created = true;
                            closest_hit = hit_distance;

                            break; // only create one anchor at a time
                        }
                    }
                }
            }

            if !new_anchor_was_created {
                if let Some((_, grand_parent_transform, grand_parent_anchor)) =
                    parent_anchor.parent.and_then(|e| anchors.get(e).ok())
                {
                    let grand_parent_entity = parent_anchor.parent.unwrap();
                    let from = child_transform.translation;
                    let to = grand_parent_transform.translation;
                    let ray_direction = (to - from).normalize();

                    let ray = Ray3d::new(from, ray_direction);
                    let mut closest_hit = (to - from).length();
                    let mut obstacle_exists = false;

                    // Check for an obstacle on path
                    for (mesh_handle, _, _, global_transform) in obstacles.iter() {
                        if let Some(mesh) = meshes.get(mesh_handle) {
                            let mesh_to_world = global_transform.compute_matrix();

                            // Check for intersection with this obstacle
                            if let Some(intersection) = ray_intersection_over_mesh(
                                mesh,
                                &mesh_to_world,
                                &ray,
                                Backfaces::Cull,
                            ) {
                                obstacle_exists = true;
                                break;
                            }
                        }
                    }

                    if !obstacle_exists {
                        //                          println!("No obstacle so removing... {:?}", parent_entity);
                        remove_anchor_event_writer.send(RemoveAnchorEvent {
                            parent: parent_entity,
                            new_parent: grand_parent_entity,
                            child: child_entity,
                        });
                    }
                }
            }
        }
    }
}

fn handle_remove_anchor(
    mut commands: Commands,
    mut remove_anchor_event_reader: EventReader<RemoveAnchorEvent>,
    mut anchors: Query<&mut Anchor>,
) {
    for event in remove_anchor_event_reader.iter() {
        if let Ok(mut child_anchor) = anchors.get_mut(event.child) {
            child_anchor.parent = Some(event.new_parent);
        }
        if let Ok(parent_anchor) = anchors.get(event.parent) {
            if let Some(leash) = parent_anchor.leash {
                commands.entity(leash).despawn_recursive();
            }
        }
        commands.entity(event.parent).despawn_recursive();
    }
}

fn print_anchors(anchors: Query<(Entity, &Anchor, &Transform)>) {
    for (e, a, t) in anchors.iter() {
        println!(
            "{:?} A: {:?} T: {} {}",
            e, a.parent, t.translation.x, t.translation.z
        );
    }
    println!("");
}

fn handle_create_anchor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut create_anchor_event_reader: EventReader<CreateAnchorEvent>,
    mut anchors: Query<&mut Anchor>,
    leashes: Query<&Leash>,
) {
    for event in create_anchor_event_reader.iter() {
        // check if parent exists?
        if let Ok(mut child_anchor) = anchors.get_mut(event.child) {
            //          println!("anchor created");
            let leash_color = if let Some(leash) = child_anchor.leash {
                                  if let Ok(leash) = leashes.get(leash) {
                                      leash.color
                                  } else {
                                      Color::PURPLE
                                  }
                              } else {
                                  Color::PURPLE
                              };

            let leash = commands
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Box::default())),
                    material: materials.add(StandardMaterial {
                        unlit: true,
                        base_color: leash_color,
                        ..Default::default()
                    }),
                    transform: Transform::from_scale(Vec3::ZERO),
                    ..Default::default()
                })
                .insert(Leash {
                    color: leash_color
                })
                .id();

            let new_anchor = commands
                .spawn_bundle(PbrBundle {
                    transform: Transform::from_translation(event.position),
                    ..Default::default()
                })
                .insert(Anchor {
                    parent: Some(event.parent),
                    leash: Some(leash),
                })
                .id();

            child_anchor.parent = Some(new_anchor);
        }
    }
}
