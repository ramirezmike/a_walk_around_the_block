use crate::{collision, player, player::PlayerAction, AppState, target, leash};
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use bevy::render::primitives::Aabb;
use bevy_mod_raycast::{
    ray_intersection_over_mesh, Backfaces, DefaultPluginState, DefaultRaycastingPlugin, Ray3d,
    RayCastMesh, RayCastMethod, RayCastSource, RaycastSystem,
};
use std::cmp::Ordering;

pub struct BotPlugin;

impl Plugin for BotPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(AppState::InGame)
                .with_system(update_bot_ai.label("ai"))
        );
    }
}

#[derive(Component)]
pub struct Bot {
    mind_cooldown: f32,
    target: Option<Vec2>,
}

impl Default for Bot {
    fn default() -> Self {
        Bot {
            mind_cooldown: 0.0,
            target: None,
        }
    }
}

impl Bot {
    pub fn can_think(&self) -> bool {
        self.mind_cooldown <= 0.0
    }
}

#[derive(Bundle)]
pub struct BotBundle {
    player: player::Player,
    bot: Bot,
    #[bundle]
    input_manager: InputManagerBundle<PlayerAction>,
}

impl BotBundle {
    pub fn new() -> Self {
        BotBundle {
            player: player::Player::new(),
            bot: Bot::default(),
            input_manager: InputManagerBundle {
                input_map: InputMap::default(),
                action_state: ActionState::default(),
            },
        }
    }
}

fn update_bot_ai(
    time: Res<Time>,
    mut bots: Query<(Entity, &mut Bot, &Transform), (Without<leash::PathObstacle>, Without<target::Target>)>,
    targets: Query<(Entity, &Transform), (With<target::Target>, Without<Bot>)>,
    obstacles: Query<
        (&Handle<Mesh>, &Transform, &Aabb, &GlobalTransform),
        (With<leash::PathObstacle>, Without<Bot>),
    >,
    meshes: Res<Assets<Mesh>>,
    mut player_move_event_writer: EventWriter<player::PlayerMoveEvent>,
) {
    for (entity, mut bot, bot_transform) in bots.iter_mut() {
        for (_, target_transform) in targets.iter() {
            let from = bot_transform.translation;
            let to = target_transform.translation;
            let ray_direction = (to - from).normalize();

            let ray = Ray3d::new(from, ray_direction);
            let mut closest_hit = (to - from).length();
            let mut obstacle_exists = false;

            for (mesh_handle, transform, aabb, global_transform) in obstacles.iter() {
                if let Some(mesh) = meshes.get(mesh_handle) {
                    let mesh_to_world = transform.compute_matrix();

                    // Check for intersection with this obstacle
                    if let Some(intersection) =
                        ray_intersection_over_mesh(mesh, &mesh_to_world, &ray, Backfaces::Cull) {
                        obstacle_exists = true;
                        break;
                    }
                }
            }

            if !obstacle_exists {
                player_move_event_writer.send(player::PlayerMoveEvent {
                    entity, 
                    movement: player::Movement::Push(ray_direction)
                });
                break;
            }
        }
    }
}
