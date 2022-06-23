use crate::{AppState, player, bot, game_state, leash, audio, assets::GameAssets, CleanupMarker};
use bevy::prelude::*;
use bevy::gltf::Gltf;
use rand::seq::SliceRandom;
use rand::thread_rng;

pub struct PickupPlugin;

impl Plugin for PickupPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PickupEvent>()
            .add_system_set(SystemSet::on_update(AppState::InGame)
                            .with_system(update_pickups)
                            .with_system(handle_pickup_event)
                            .with_system(animate_pickups)
                            );
    }
}

pub struct PickupEvent {
    entity: Entity,
    pickup_type: PickupType 
}

#[derive(Component)]
pub struct Pickup {
    pub pickup_type: PickupType
}

impl Pickup {
    fn new(pet_type: bot::PetType) -> Self {
        Pickup {
            pickup_type: PickupType::Pet(pet_type)
        }
    }
}

#[derive(Clone, Copy)]
pub enum PickupType {
    Pet(bot::PetType),
    Coin
}

pub fn make_random_pet() -> (Pickup, String) {
    let mut rng = thread_rng();
    let pet_types = vec!(bot::PetType::Chicken, bot::PetType::Dog, bot::PetType::ChickenDog);
    let picked_pet_type = pet_types.choose(&mut rng).unwrap_or(&bot::PetType::Chicken);
    let pickup = Pickup::new(*picked_pet_type);

    match picked_pet_type {
        bot::PetType::Chicken => (pickup, "models/chicken.glb#Scene0".to_string()),
        bot::PetType::Dog => (pickup, "models/dog.glb#Scene0".to_string()),
        bot::PetType::ChickenDog => (pickup, "models/chickendog.glb#Scene0".to_string()),
    }
}

fn handle_pickup_event( 
    mut commands: Commands,
    mut pickup_event_reader: EventReader<PickupEvent>,
    mut audio: audio::GameAudio,
    game_assets: Res<GameAssets>,
    asset_server: Res<AssetServer>,
    assets_gltf: Res<Assets<Gltf>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut game_state: ResMut<game_state::GameState>,
    mut players: Query<(Entity, &mut player::Player, &Transform), Without<bot::Bot>>,
) {
    for event in pickup_event_reader.iter() {
        commands.entity(event.entity).despawn_recursive();
        audio.play_sfx(&game_assets.pickup);

        match event.pickup_type {
            PickupType::Coin => {
                game_state.score += 10;
            },
            PickupType::Pet(pet) => {
                if let Ok((player_entity, mut player, player_transform)) = players.get_single_mut() {
                    if !player.looking_for_pets() {
                        continue;
                    }

                    let leash_color = player.get_next_leash_color();

                    let model = match pet {
                                    bot::PetType::Chicken => "models/chicken.glb#Scene0",
                                    bot::PetType::Dog => "models/dog.glb#Scene0",
                                    bot::PetType::ChickenDog => "models/chickendog.glb#Scene0",
                                };

                    if let Some(_) = assets_gltf.get(&game_assets.chicken) {
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
                            .insert(leash::Leash {
                                color: leash_color
                            })
                            .insert(CleanupMarker)
                            .id();

                        let chicken_id = commands
                            .spawn_bundle((
                                Transform::from_translation(player_transform.translation),
                                GlobalTransform::identity(),
                            ))
                            .with_children(|parent| {
                                parent
                                    .spawn_bundle((
                                        Transform::from_rotation(Quat::from_rotation_y(
                                            std::f32::consts::FRAC_PI_2,
                                        )),
                                        GlobalTransform::identity(),
                                    ))
                                    .with_children(|parent| {
                                        parent.spawn_scene(asset_server.load(model));
                                    });
                            })
                            .insert_bundle(bot::BotBundle::new())
                            .insert(CleanupMarker)
                            .insert(bot::Pet {
                                pet_type: bot::PetType::Chicken,
                            })
                            .insert(leash::Anchor {
                                parent: Some(player_entity),
                                leash: Some(leash),
                            })
                            .id();
                        player.add_pet(chicken_id);
                    }
                }
            },
        }
    }
}

fn update_pickups(
    time: Res<Time>,
    mut cooldown: Local<f32>,
    pickups: Query<(Entity, &Transform, &Pickup)>,
    players: Query<&Transform, (With<player::Player>, Without<bot::Bot>)>,
    game_state: Res<game_state::GameState>,
    mut pickup_event_writer: EventWriter<PickupEvent>,
) {
    // handling cool down
    *cooldown -= time.delta_seconds();
    *cooldown = cooldown.clamp(-10.0, 2.0);

    if *cooldown <= 0.0 {
        if let Ok(player_transform) = players.get_single() {
            for (entity, pickup_transform, pickup) in pickups.iter() {
                if game_state::map_to_chunk(pickup_transform.translation) == game_state.current_chunk 
                && player_transform.translation.distance(pickup_transform.translation) < 1.5 {
                    pickup_event_writer.send(PickupEvent {
                        entity,
                        pickup_type: pickup.pickup_type
                    });
                }
            }
        }
        *cooldown = 0.2;
    }
}

fn animate_pickups(mut pickups: Query<&mut Transform, With<Pickup>>, time: Res<Time>) {
    for mut transform in pickups.iter_mut() {
        transform.rotate(Quat::from_rotation_y(time.delta_seconds()));
        transform.scale = Vec3::splat(
            1.0 + (1.0 / 2.0 * time.seconds_since_startup().sin() as f32).abs(),
        );
    }
}
