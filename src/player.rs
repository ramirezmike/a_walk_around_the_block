use crate::{bot, collision, direction, leash, AppState, game_state, ingame_ui};
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use rand::Rng;
use std::collections::HashMap;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<PlayerAction>::default())
            .add_event::<PlayerMoveEvent>()
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .with_system(handle_input.label("input"))
                    .with_system(move_player.label("move_player").after("input")),
            );
    }
}

pub trait ZeroSignum {
    fn zero_signum(&self) -> Vec3;
}

impl ZeroSignum for Vec3 {
    fn zero_signum(&self) -> Vec3 {
        let convert = |n| {
            if n < 0.1 && n > -0.1 {
                0.0
            } else if n > 0.0 {
                1.0
            } else {
                -1.0
            }
        };

        Vec3::new(convert(self.x), convert(self.y), convert(self.z))
    }
}

fn move_player(
    time: Res<Time>,
    mut players: Query<(Entity, &mut Transform, &mut Player)>,
    mut player_move_event_reader: EventReader<PlayerMoveEvent>,
    collidables: collision::Collidables,
) {
    let mut move_events = HashMap::new();
    for move_event in player_move_event_reader.iter() {
        move_events.entry(move_event.entity).or_insert(move_event);
    }

    for (entity, mut transform, mut player) in players.iter_mut() {
        let speed: f32 = player.speed;
        let rotation_speed: f32 = player.rotation_speed;
        let friction: f32 = player.friction;

        player.velocity *= friction.powf(time.delta_seconds());
        let mut yank_strength = 1.0;
        if let Some(move_event) = move_events.get(&entity) {
            match move_event.movement {
                Movement::Normal(direction) => {
                    let acceleration = Vec3::from(direction);
                    player.velocity += (acceleration.zero_signum() * speed) * time.delta_seconds();
                }
                Movement::Yank(direction, strength) => {
                    let acceleration = Vec3::from(direction);
                    yank_strength = strength; 
                    player.velocity += (acceleration.zero_signum() * speed * strength) * time.delta_seconds();
                }
                Movement::Pull(direction) => {
                    let acceleration = direction;
                    player.velocity += (acceleration.zero_signum() * speed) * time.delta_seconds();
                }
                Movement::Push(direction) => {
                    let acceleration = direction;
                    player.velocity += (acceleration.zero_signum() * speed) * time.delta_seconds();
                }
            }
        }

        player.velocity = player.velocity.clamp_length_max(speed * yank_strength);

        let mut new_translation = transform.translation + (player.velocity * time.delta_seconds());
        collidables.fit_in(
            &transform.translation,
            &mut new_translation,
            &mut player.velocity,
        );

        let angle = (-(new_translation.z - transform.translation.z))
            .atan2(new_translation.x - transform.translation.x);
        let rotation = Quat::from_axis_angle(Vec3::Y, angle);
        transform.translation = new_translation;

        if player.velocity.length() > 1.0 {
            let bobbing_velocity = (time.seconds_since_startup() as f32
                * (2.0 * std::f32::consts::PI)
                * 4.0
                * player.random)
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
        if !new_rotation.is_nan() && player.velocity.length() > 0.5 {
            transform.rotation = rotation;
        }

        // make the player all squishy like
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

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum PlayerAction {
    Up,
    Down,
    Left,
    Right,

    ActionUp,
    ActionDown,
    ActionLeft,
    ActionRight,
    Pause,
}

impl PlayerAction {
    const DIRECTIONS: [Self; 4] = [
        PlayerAction::Up,
        PlayerAction::Down,
        PlayerAction::Left,
        PlayerAction::Right,
    ];

    fn direction(self) -> direction::Direction {
        match self {
            PlayerAction::Up => direction::Direction::UP,
            PlayerAction::Down => direction::Direction::DOWN,
            PlayerAction::Left => direction::Direction::LEFT,
            PlayerAction::Right => direction::Direction::RIGHT,
            _ => direction::Direction::NEUTRAL,
        }
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Player {
    pub velocity: Vec3,
    pub speed: f32,
    pub rotation_speed: f32,
    pub friction: f32,
    pub random: f32,

    pub north_pet: Option<Entity>,
    pub south_pet: Option<Entity>,
    pub west_pet: Option<Entity>,
    pub east_pet: Option<Entity>,
}

impl Player {
    pub fn new(starting_pet: Option<Entity>) -> Self {
        let mut rng = rand::thread_rng();

        Player {
            velocity: Vec3::default(),
            speed: 40.0,
            rotation_speed: 1.0,
            friction: 0.01,
            random: rng.gen_range(0.5..1.0),
            north_pet: None,
            south_pet: starting_pet,
            west_pet: None,
            east_pet: None,
        }
    }

    pub fn add_pet(&mut self, pet: Entity) {
        if self.south_pet.is_none() {
            self.south_pet = Some(pet);
            return;
        }
        if self.east_pet.is_none() {
            self.east_pet = Some(pet);
            return;
        }
        if self.west_pet.is_none() {
            self.west_pet = Some(pet);
            return;
        }
        if self.north_pet.is_none() {
            self.north_pet = Some(pet);
            return;
        }
    }

    pub fn get_next_leash_color(&self) -> Color {
        if self.south_pet.is_none() {
            Color::GREEN
        } else if self.east_pet.is_none() {
            Color::RED
        } else if self.west_pet.is_none() {
            Color::BLUE
        } else {
            Color::YELLOW
        }
    }

    pub fn looking_for_pets(&self) -> bool {
        self.north_pet.is_none()
        ||
        self.south_pet.is_none()
        ||
        self.west_pet.is_none()
        ||
        self.east_pet.is_none()
    }
}

#[derive(Bundle)]
pub struct PlayerBundle {
    player: Player,
    #[bundle]
    input_manager: InputManagerBundle<PlayerAction>,
}

impl PlayerBundle {
    pub fn new(starting_pet: Option<Entity>) -> Self {
        PlayerBundle {
            player: Player::new(starting_pet),
            input_manager: InputManagerBundle {
                input_map: PlayerBundle::default_input_map(),
                action_state: ActionState::default(),
            },
        }
    }

    fn default_input_map() -> InputMap<PlayerAction> {
        use PlayerAction::*;
        let mut input_map = InputMap::default();

        input_map.set_gamepad(Gamepad(0));

        // Movement
        input_map.insert(Up, KeyCode::Up);
        input_map.insert(Up, KeyCode::W);
        input_map.insert(Up, GamepadButtonType::DPadUp);

        input_map.insert(Down, KeyCode::Down);
        input_map.insert(Down, KeyCode::S);
        input_map.insert(Down, GamepadButtonType::DPadDown);

        input_map.insert(Left, KeyCode::Left);
        input_map.insert(Left, KeyCode::A);
        input_map.insert(Left, GamepadButtonType::DPadLeft);

        input_map.insert(Right, KeyCode::Right);
        input_map.insert(Right, KeyCode::D);
        input_map.insert(Right, GamepadButtonType::DPadRight);

        // Actions
        input_map.insert(ActionUp, KeyCode::I);
        input_map.insert(ActionUp, GamepadButtonType::North);

        input_map.insert(ActionDown, KeyCode::K);
        input_map.insert(ActionDown, GamepadButtonType::South);

        input_map.insert(ActionLeft, KeyCode::J);
        input_map.insert(ActionLeft, GamepadButtonType::West);

        input_map.insert(ActionRight, KeyCode::L);
        input_map.insert(ActionRight, GamepadButtonType::East);

        // Other
        input_map.insert(Pause, KeyCode::Escape);

        input_map
    }
}

pub struct PlayerMoveEvent {
    pub entity: Entity,
    pub movement: Movement,
}

pub enum Movement {
    Normal(direction::Direction),
    Pull(Vec3),
    Yank(Vec3, f32), //direction, strength
    Push(Vec3),
}

fn handle_input(
    mut app_state: ResMut<State<AppState>>,
    players: Query<(Entity, &ActionState<PlayerAction>, &Transform, &Player), Without<bot::Bot>>,
    anchors: Query<&Transform, With<leash::Anchor>>,
    pets: Query<(&Transform, &leash::Anchor), With<bot::Pet>>,
    game_state: Res<game_state::GameState>,
    mut player_move_event_writer: EventWriter<PlayerMoveEvent>,
    mut button_pressed_event_writer: EventWriter<ingame_ui::ButtonPressedEvent>,
    mut button_hold_event_writer: EventWriter<ingame_ui::ButtonHoldEvent>,
) {
    for (entity, action_state, transform, player) in players.iter() {
        //println!("T: {:?}", transform.translation);
        let mut direction = direction::Direction::NEUTRAL;

        for input_direction in PlayerAction::DIRECTIONS {
            if action_state.pressed(input_direction) {
                direction += input_direction.direction();
            }
        }

        if direction != direction::Direction::NEUTRAL {
            player_move_event_writer.send(PlayerMoveEvent {
                entity,
                movement: Movement::Normal(direction),
            });
        }

        if action_state.just_pressed(PlayerAction::Pause) {
            app_state.push(AppState::Pause).unwrap();
        }

        if action_state.just_pressed(PlayerAction::ActionUp) {
            if let Some(pet) = player.north_pet {
                let (pet_transform, pet_anchor) = pets.get(pet).unwrap();
                if let Some(pet_parent) = pet_anchor.parent {
                    if let Ok(anchor_transform) = anchors.get(pet_parent) {
                        let pull_direction =
                            anchor_transform.translation - pet_transform.translation;
                        player_move_event_writer.send(PlayerMoveEvent {
                            entity: pet,
                            movement: Movement::Yank(pull_direction, game_state.yank_strength),
                        });
                    }
                }
            }

            button_pressed_event_writer.send(ingame_ui::ButtonPressedEvent {
                button_type: ingame_ui::LeashButtonType::Yellow
            });
        }
        if action_state.pressed(PlayerAction::ActionUp) {
            if let Some(pet) = player.north_pet {
                let (pet_transform, pet_anchor) = pets.get(pet).unwrap();
                if let Some(pet_parent) = pet_anchor.parent {
                    if let Ok(anchor_transform) = anchors.get(pet_parent) {
                        let pull_direction =
                            anchor_transform.translation - pet_transform.translation;
                        player_move_event_writer.send(PlayerMoveEvent {
                            entity: pet,
                            movement: Movement::Pull(pull_direction),
                        });
                    }
                }
            }

            button_hold_event_writer.send(ingame_ui::ButtonHoldEvent {
                button_type: ingame_ui::LeashButtonType::Yellow
            });
        }

        if action_state.just_pressed(PlayerAction::ActionDown) {
            if let Some(pet) = player.south_pet {
                let (pet_transform, pet_anchor) = pets.get(pet).unwrap();
                if let Some(pet_parent) = pet_anchor.parent {
                    if let Ok(anchor_transform) = anchors.get(pet_parent) {
                        let pull_direction =
                            anchor_transform.translation - pet_transform.translation;
                        player_move_event_writer.send(PlayerMoveEvent {
                            entity: pet,
                            movement: Movement::Yank(pull_direction, game_state.yank_strength),
                        });
                    }
                }
            }

            button_pressed_event_writer.send(ingame_ui::ButtonPressedEvent {
                button_type: ingame_ui::LeashButtonType::Green
            });
        }

        if action_state.pressed(PlayerAction::ActionDown) {
            if let Some(pet) = player.south_pet {
                let (pet_transform, pet_anchor) = pets.get(pet).unwrap();
                if let Some(pet_parent) = pet_anchor.parent {
                    if let Ok(anchor_transform) = anchors.get(pet_parent) {
                        let pull_direction =
                            anchor_transform.translation - pet_transform.translation;
                        player_move_event_writer.send(PlayerMoveEvent {
                            entity: pet,
                            movement: Movement::Pull(pull_direction),
                        });
                    }
                }
            }

            button_hold_event_writer.send(ingame_ui::ButtonHoldEvent {
                button_type: ingame_ui::LeashButtonType::Green
            });
        }

        if action_state.just_pressed(PlayerAction::ActionLeft) {
            if let Some(pet) = player.west_pet {
                let (pet_transform, pet_anchor) = pets.get(pet).unwrap();
                if let Some(pet_parent) = pet_anchor.parent {
                    if let Ok(anchor_transform) = anchors.get(pet_parent) {
                        let pull_direction =
                            anchor_transform.translation - pet_transform.translation;
                        player_move_event_writer.send(PlayerMoveEvent {
                            entity: pet,
                            movement: Movement::Yank(pull_direction, game_state.yank_strength),
                        });
                    }
                }
            }

            button_pressed_event_writer.send(ingame_ui::ButtonPressedEvent {
                button_type: ingame_ui::LeashButtonType::Blue
            });
        }

        if action_state.pressed(PlayerAction::ActionLeft) {
            if let Some(pet) = player.west_pet {
                let (pet_transform, pet_anchor) = pets.get(pet).unwrap();
                if let Some(pet_parent) = pet_anchor.parent {
                    if let Ok(anchor_transform) = anchors.get(pet_parent) {
                        let pull_direction =
                            anchor_transform.translation - pet_transform.translation;
                        player_move_event_writer.send(PlayerMoveEvent {
                            entity: pet,
                            movement: Movement::Pull(pull_direction),
                        });
                    }
                }
            }

            button_hold_event_writer.send(ingame_ui::ButtonHoldEvent {
                button_type: ingame_ui::LeashButtonType::Blue
            });
        }

        if action_state.just_pressed(PlayerAction::ActionRight) {
            if let Some(pet) = player.east_pet {
                let (pet_transform, pet_anchor) = pets.get(pet).unwrap();
                if let Some(pet_parent) = pet_anchor.parent {
                    if let Ok(anchor_transform) = anchors.get(pet_parent) {
                        let pull_direction =
                            anchor_transform.translation - pet_transform.translation;
                        player_move_event_writer.send(PlayerMoveEvent {
                            entity: pet,
                            movement: Movement::Yank(pull_direction, game_state.yank_strength),
                        });
                    }
                }
            }

            button_pressed_event_writer.send(ingame_ui::ButtonPressedEvent {
                button_type: ingame_ui::LeashButtonType::Red
            });
        }

        if action_state.pressed(PlayerAction::ActionRight) {
            if let Some(pet) = player.east_pet {
                let (pet_transform, pet_anchor) = pets.get(pet).unwrap();
                if let Some(pet_parent) = pet_anchor.parent {
                    if let Ok(anchor_transform) = anchors.get(pet_parent) {
                        let pull_direction =
                            anchor_transform.translation - pet_transform.translation;
                        player_move_event_writer.send(PlayerMoveEvent {
                            entity: pet,
                            movement: Movement::Pull(pull_direction),
                        });
                    }
                }
            }

            button_hold_event_writer.send(ingame_ui::ButtonHoldEvent {
                button_type: ingame_ui::LeashButtonType::Red
            });
        }
    }
}
