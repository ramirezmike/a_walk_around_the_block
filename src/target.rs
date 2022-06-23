use crate::{AppState, collision, player::ZeroSignum};
use bevy::prelude::*;
use rand::Rng;
use std::collections::HashMap;

pub struct TargetPlugin;

impl Plugin for TargetPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TargetMoveEvent>()
            .add_system_set(SystemSet::on_update(AppState::InGame).with_system(update_targets).with_system(update_target_minds));
    }
}

pub struct TargetMoveEvent {
    pub entity: Entity,
    pub direction: Vec2,
}

#[derive(Component)]
pub struct Target {
    pub velocity: Vec3,
    pub speed: f32,
    pub rotation_speed: f32,
    pub friction: f32,
    pub random: f32,
    pub target_type: TargetType,
    pub mind_cooldown: f32,
    pub heading_to: Option::<Vec2>,
}

impl Target {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();

        Target {
            velocity: Vec3::default(),
            speed: 10.0,
            rotation_speed: 1.0,
            friction: 0.01,
            random: rng.gen_range(0.5..1.0),
            target_type: TargetType::Bug,
            heading_to: None,
            mind_cooldown: 0.0
        }
    }

    pub fn can_think(&self) -> bool {
        self.mind_cooldown <= 0.0
    }
}

pub fn make_random_target() -> (Target, String) {
    (Target::new(), "models/person.glb#Scene0".to_string()) 
}


pub enum TargetType {
    Bug
}

fn update_targets(
    time: Res<Time>,
    mut targets: Query<(Entity, &mut Transform, &mut Target)>,
    mut target_move_event_reader: EventReader<TargetMoveEvent>,
    collidables: collision::Collidables,
) {
    let mut move_events = HashMap::new();
    for move_event in target_move_event_reader.iter() {
        move_events.entry(move_event.entity).or_insert(move_event);
    }

    for (entity, mut transform, mut target) in targets.iter_mut() {
        let speed: f32 = target.speed;
        let rotation_speed: f32 = target.rotation_speed;
        let friction: f32 = target.friction;

        target.velocity *= friction.powf(time.delta_seconds());
        if let Some(move_event) = move_events.get(&entity) {
            let acceleration = Vec3::new(move_event.direction.x, 0.0, move_event.direction.y);
            target.velocity += (acceleration.zero_signum() * speed) * time.delta_seconds();
        }

        target.velocity = target.velocity.clamp_length_max(speed);

        let mut new_translation = transform.translation + (target.velocity * time.delta_seconds());
        collidables.fit_in(
            &transform.translation,
            &mut new_translation,
            &mut target.velocity,
        );

        let angle = (-(new_translation.z - transform.translation.z))
            .atan2(new_translation.x - transform.translation.x);
        let rotation = Quat::from_axis_angle(Vec3::Y, angle);
        transform.translation = new_translation;

        if target.velocity.length() > 1.0 {
            let bobbing_velocity = (time.seconds_since_startup() as f32
                * (2.0 * std::f32::consts::PI)
                * 4.0
                * target.random)
                .sin() as f32;
            transform.translation.y += bobbing_velocity * (time.delta_seconds() * 4.0);
        //          transform.rotate(Quat::from_rotation_x(
        //              bobbing_velocity * (time.delta_seconds() * 8.0),
        //          ));
        } else {
            transform.translation.y += -4.0 * time.delta_seconds(); // gravity
        }
        transform.translation.y = transform.translation.y.clamp(0.0, 0.5);

        let new_rotation = transform
            .rotation
            .lerp(rotation, time.delta_seconds() * rotation_speed);

        // don't rotate if we're not moving or if uhh rotation isnt a number?? why isn't it a number? who did this
        if !new_rotation.is_nan() && target.velocity.length() > 0.5 {
            transform.rotation = rotation;
        }

        // make the target all squishy like
        if transform.scale.x != 1.0 || transform.scale.y != 1.0 {
            let new_scale = transform
                .scale
                .lerp(Vec3::new(1.0, 1.0, 1.0), time.delta_seconds() * 4.0);
            if new_scale.is_nan() || transform.scale.distance(new_scale) < 0.0001 {
                transform.scale = Vec3::new(1.0, 1.0, 1.0);
            } else {
                transform.scale = new_scale;
            }
        }
    }
}

fn update_target_minds(
    time: Res<Time>,
    mut targets: Query<(Entity, &mut Transform, &mut Target)>,
    mut target_move_event_writer: EventWriter<TargetMoveEvent>,
) {
    for (entity, mut transform, mut target) in targets.iter_mut() {
        // handling mind cool down
        target.mind_cooldown -= time.delta_seconds();
        target.mind_cooldown = target.mind_cooldown.clamp(-10.0, 30.0);

        if let Some(heading_to) = target.heading_to {
            target_move_event_writer.send(TargetMoveEvent {
                entity,
                direction: heading_to,
            });
        }

        if !target.can_think() {
            continue;
        }

        let random_direction = get_random_direction();
        target.heading_to = Some(random_direction);
        target.mind_cooldown = 2.0;
    }
}

pub fn get_random_direction() -> Vec2 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let x: f32 = rng.gen_range(-100.0..100.0);
    let z: f32 = rng.gen_range(-100.0..100.0);

    Vec2::new(x, z).normalize()
}
