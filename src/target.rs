use crate::{AppState, collision, player, player::ZeroSignum, follow_text, bot, game_state, audio, assets::GameAssets};
use bevy::prelude::*;
use rand::Rng;
use bevy::gltf::Gltf;
use std::collections::HashMap;
use rand::seq::SliceRandom;
use rand::thread_rng;

pub struct TargetPlugin;

impl Plugin for TargetPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TargetMoveEvent>()
            .add_event::<TargetHitEvent>()
            .add_system_set(SystemSet::on_update(AppState::InGame).with_system(update_targets)
                            .with_system(handle_target_hit_event)
                            .with_system(update_target_minds));
    }
}

pub struct TargetMoveEvent {
    pub entity: Entity,
    pub direction: Vec2,
}

pub struct TargetHitEvent {
    pub entity: Entity,
    pub hit_by: bot::PetType
}

fn handle_target_hit_event(
    mut commands: Commands,
    mut target_hit_event_reader: EventReader<TargetHitEvent>,
    mut follow_text_event_writer: EventWriter<follow_text::FollowTextEvent>,
    mut targets: Query<(Entity, &mut Target, &Transform)>,
    players: Query<Entity, (With<player::Player>, Without<bot::Bot>)>,
    mut game_state: ResMut<game_state::GameState>,
    mut audio: audio::GameAudio,
    game_assets: Res<GameAssets>,
) { 
    for event in target_hit_event_reader.iter() {
        if let Ok(player_entity) = players.get_single() {
            if let Ok((target_entity, mut target, target_transform)) = targets.get_mut(event.entity) {
                match target.hit_and_response(event.hit_by, &mut audio, &game_assets, &game_state) {
                    TargetHitResponse::Text(text, color, ttl) => {
                        follow_text_event_writer.send(follow_text::FollowTextEvent {
                            follow: follow_text::FollowThing::Entity(event.entity),
                            text: text,
                            color: color,
                            time_to_live: ttl,
                        });
                    },
                    TargetHitResponse::ScoreUp(text, score, color, ttl, death) => {
                        game_state.score += score;
                        if death {
                            commands.entity(target_entity).despawn_recursive();
                            follow_text_event_writer.send(follow_text::FollowTextEvent {
                                follow: follow_text::FollowThing::Spot(target_transform.translation),
                                text: text,
                                color: color,
                                time_to_live: ttl,
                            });
                        } else {
                            follow_text_event_writer.send(follow_text::FollowTextEvent {
                                follow: follow_text::FollowThing::Entity(event.entity),
                                text: text,
                                color: color,
                                time_to_live: ttl,
                            });

                            follow_text_event_writer.send(follow_text::FollowTextEvent {
                                follow: follow_text::FollowThing::Entity(player_entity),
                                text: format!("+{}", score),
                                color: Color::GREEN,
                                time_to_live: 2.0,
                            });
                        }
                    },
                    TargetHitResponse::ScoreDown(text, score, color, ttl, death) => {
                        game_state.score = game_state.score.saturating_sub(score);
                        if death {
                            commands.entity(target_entity).despawn_recursive();
                            follow_text_event_writer.send(follow_text::FollowTextEvent {
                                follow: follow_text::FollowThing::Spot(target_transform.translation),
                                text: text,
                                color: color,
                                time_to_live: ttl,
                            });

                            follow_text_event_writer.send(follow_text::FollowTextEvent {
                                follow: follow_text::FollowThing::Entity(player_entity),
                                text: format!("-{}", score),
                                color: Color::RED,
                                time_to_live: 2.0,
                            });
                        } else {
                            follow_text_event_writer.send(follow_text::FollowTextEvent {
                                follow: follow_text::FollowThing::Entity(event.entity),
                                text: text,
                                color: color,
                                time_to_live: ttl,
                            });

                            follow_text_event_writer.send(follow_text::FollowTextEvent {
                                follow: follow_text::FollowThing::Entity(player_entity),
                                text: format!("-{}", score),
                                color: Color::RED,
                                time_to_live: 2.0,
                            });
                        }
                    },
                    TargetHitResponse::Nothing => ()
                }
            }
        }
    }
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
    pub hit_cooldown: f32,
    pub health: f32,
    pub heading_to: Option::<Vec2>,
    pub ignore: isize,
}

impl Target {
    pub fn new(target_type: TargetType) -> Self {
        let mut rng = rand::thread_rng();

        match target_type {
            TargetType::Person => {
                Target {
                    velocity: Vec3::default(),
                    speed: 10.0,
                    rotation_speed: 1.0,
                    friction: 0.01,
                    random: rng.gen_range(0.5..1.0),
                    target_type: target_type,
                    heading_to: None,
                    mind_cooldown: 0.0,
                    hit_cooldown: 0.0,
                    health: 5.0,
                    ignore: 2,
                }
            },
            TargetType::Worm => {
                Target {
                    velocity: Vec3::default(),
                    speed: 1.0,
                    rotation_speed: 1.0,
                    friction: 0.01,
                    random: rng.gen_range(0.5..1.0),
                    target_type: target_type,
                    heading_to: None,
                    mind_cooldown: 0.0,
                    hit_cooldown: 0.0,
                    health: 1.0,
                    ignore: 1,
                }
            },
            TargetType::Chip => {
                Target {
                    velocity: Vec3::default(),
                    speed: 12.0,
                    rotation_speed: 1.0,
                    friction: 0.01,
                    random: rng.gen_range(0.5..1.0),
                    target_type: target_type,
                    heading_to: None,
                    mind_cooldown: 0.0,
                    hit_cooldown: 0.0,
                    health: 2.5,
                    ignore: 1,
                }
            },
        }
    }


    pub fn hit_and_response(&mut self, 
        hit_by: bot::PetType,
        mut audio: &mut audio::GameAudio,
        game_assets: &Res<GameAssets>,
        game_state: &ResMut<game_state::GameState>,
    ) -> TargetHitResponse {
        // here we go!!
        if self.hit_cooldown >= 0.0 {
            return TargetHitResponse::Nothing;
        }

        self.hit_cooldown = 1.0 / game_state.game_speed;
        let standard_time = 2.0;

        match self.target_type {
            TargetType::Person => {
                match hit_by {
                    bot::PetType::Dog => {
                        self.ignore -= 1 * (game_state.game_speed as isize);
                        if self.ignore <= 0 {
                            audio.play_sfx(&game_assets.powerup);
                            TargetHitResponse::ScoreUp(get_happy_dog_msg(), 50, Color::GREEN, standard_time, false)
                        } else {
                            TargetHitResponse::Nothing
                        }
                    },
                    bot::PetType::Chicken => {
                        TargetHitResponse::ScoreDown(get_angry_chicken_msg(), 50, Color::RED, standard_time, false)
                    },
                    bot::PetType::ChickenDog => {
                        self.health -= 1.0 * game_state.game_speed;

                        audio.play_sfx(&game_assets.attack);
                        if self.health <= 0.0 {
                            TargetHitResponse::ScoreUp("+100".to_string(), 100, Color::GREEN, standard_time, true)
                        } else {
                            TargetHitResponse::Text(get_chicken_dog_msg(), Color::RED, standard_time)
                        }
                    },
                }
            },
            TargetType::Chip => {
                match hit_by {
                    bot::PetType::Dog => {
                        self.health -= 1.0 * game_state.game_speed;

                        audio.play_sfx(&game_assets.attack);
                        if self.health <= 0.0 {
                            TargetHitResponse::ScoreDown("*squeak*".to_string(), 100, Color::RED, standard_time, true)
                        } else {
                            TargetHitResponse::Text("*sad chipmunk noise*".to_string(), Color::RED, standard_time)
                        }
                    },
                    bot::PetType::Chicken => TargetHitResponse::Nothing,
                    bot::PetType::ChickenDog => {
                        self.health -= 1.0 * game_state.game_speed;
                        audio.play_sfx(&game_assets.attack);

                        TargetHitResponse::ScoreUp("+50".to_string(), 50, Color::GREEN, standard_time, self.health <= 0.0)
                    },
                }
            },
            TargetType::Worm => {
                match hit_by {
                    bot::PetType::Dog => TargetHitResponse::Nothing,
                    bot::PetType::Chicken | bot::PetType::ChickenDog => {
                        self.health -= 1.0 * game_state.game_speed;
                        audio.play_sfx(&game_assets.attack);

                        if self.health <= 0.0 {
                            TargetHitResponse::ScoreUp("+50".to_string(), 50, Color::GREEN, standard_time, true)
                        } else {
                            TargetHitResponse::Text("*wormy noises*".to_string(), Color::DARK_GREEN, standard_time)
                        }
                    },
                }
            },
        }
    }

    pub fn can_think(&self) -> bool {
        self.mind_cooldown <= 0.0
    }
}

pub fn make_random_target(game_assets: &Res<GameAssets>) -> (Target, Handle<Gltf>) {
    let mut rng = thread_rng();
    let types = vec!(TargetType::Person, TargetType::Worm, TargetType::Chip);
    let picked_type = types.choose(&mut rng).unwrap_or(&TargetType::Person);
    let target = Target::new(*picked_type);

    match picked_type {
        TargetType::Person => (target, game_assets.get_random_player_model()),
        TargetType::Worm => (target, game_assets.worm.clone()),
        TargetType::Chip => (target, game_assets.chip.clone()),
    }
}


#[derive(Copy, Clone, PartialEq)]
pub enum TargetType {
    Person,
    Worm,
    Chip
}

pub enum TargetHitResponse {
    Text(String, Color, f32),
    ScoreUp(String, usize, Color, f32, bool),
    ScoreDown(String, usize, Color, f32, bool),
    Nothing,
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
        target.hit_cooldown -= time.delta_seconds();
        target.hit_cooldown = target.hit_cooldown.clamp(-10.0, 30.0);

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

pub fn get_angry_chicken_msg() -> String {
    let mut rng = thread_rng();
    let messages = vec!(
        "get away!",
        "leave me alone!",
        "stop it!",
        "Uhhhhhh",
        "ew!",
        "can you not?",
        "what's that smell?",
        "seriously, a chicken??",
        "please stop",
        "ok.. are you done?",
        "get off me!",
        "don't do that!",
        "This is unpleasant",
        "I didn't ask for this",
        "I don't like this",
        "no!",
    );

    messages.choose(&mut rng).unwrap_or(&"Ahh").to_string()
}

pub fn get_chicken_dog_msg() -> String {
    let mut rng = thread_rng();
    let messages = vec!(
        "WHAT IS THAT",
        "OH MY GOD",
        "HELP!!",
        "IT'S EATING ME",
        "SAVE ME PLEASE!",
        "*CRUNCHING SOUNDS*",
        "MAKE IT STOP",
        "OH NO, NOT AGAIN",
        "I DON'T DESERVE THIS",
        "WHY ME",
        "NO NO NO",
        "AHHHHHHH!",
        "IS THIS REAL",
    );

    messages.choose(&mut rng).unwrap_or(&"AHHH").to_string()
}

pub fn get_happy_dog_msg() -> String {
    let mut rng = thread_rng();
    let messages = vec!(
        "What a cute puppy",
        "I'm happy now",
        "That was great",
        "Thank you!",
        "Have a nice day",
        "Sweet!",
        "What a soft pupper",
        "haha, neat",
        "I petted that dog",
        "What a treat!",
        "Aww, that was nice",
        "You're the best!",
    );

    messages.choose(&mut rng).unwrap_or(&"Aww!").to_string()
}
