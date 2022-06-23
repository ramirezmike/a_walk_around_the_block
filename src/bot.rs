use crate::{collision, leash, player, player::PlayerAction, target, AppState};
use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use bevy_mod_raycast::{
    ray_intersection_over_mesh, Backfaces, DefaultPluginState, DefaultRaycastingPlugin, Ray3d,
    RayCastMesh, RayCastMethod, RayCastSource, RaycastSystem,
};
use leafwing_input_manager::prelude::*;
use std::cmp::Ordering;

pub struct BotPlugin;

impl Plugin for BotPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(AppState::InGame).with_system(update_bot_ai.label("ai")),
        );
    }
}

#[derive(Component)]
pub struct Bot {
    mind_cooldown: f32,
    target: Option<Vec3>,
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
        self.mind_cooldown <= 0.0 && self.target.is_none()
    }
}

#[derive(Component)]
pub struct Pet {
    pub pet_type: PetType,
}

#[derive(Copy, Clone)]
pub enum PetType {
    Chicken,
    Dog,
    ChickenDog,
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
            player: player::Player::new(None),
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
    mut bots: Query<
        (Entity, &mut Bot, &Transform),
        (Without<leash::PathObstacle>, Without<target::Target>),
    >,
    targets: Query<(Entity, &Transform), (With<target::Target>, Without<Bot>)>,
    player: Query<&Transform, (With<player::Player>, Without<Bot>)>,
    obstacles: Query<
        (&Handle<Mesh>, &Transform, &Aabb, &GlobalTransform),
        (With<leash::PathObstacle>, Without<Bot>),
    >,
    meshes: Res<Assets<Mesh>>,
    mut player_move_event_writer: EventWriter<player::PlayerMoveEvent>,
) {
    for (entity, mut bot, bot_transform) in bots.iter_mut() {
        // handling mind cool down
        bot.mind_cooldown -= time.delta_seconds();
        bot.mind_cooldown = bot.mind_cooldown.clamp(-10.0, 30.0);

        bot.target = None;

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
                        ray_intersection_over_mesh(mesh, &mesh_to_world, &ray, Backfaces::Cull)
                    {
                        obstacle_exists = true;
                        break;
                    }
                }
            }

            if !obstacle_exists {
                bot.target = Some(ray_direction);
                break;
            }
        }

        if let Some(target) = bot.target {
            player_move_event_writer.send(player::PlayerMoveEvent {
                entity,
                movement: player::Movement::Push(target),
            });
        }

        if !bot.can_think() {
            continue;
        }

        if let Ok(player) = player.get_single() {
            if player.translation.distance(bot_transform.translation) > 3.0 {
                player_move_event_writer.send(player::PlayerMoveEvent {
                    entity,
                    movement: player::Movement::Push(player.translation - bot_transform.translation),
                });
            }
        }
    }
}
