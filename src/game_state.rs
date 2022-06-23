use crate::{assets::GameAssets, bot, component_adder, pickup, player, AppState, CleanupMarker, target};
use bevy::gltf::Gltf;
use bevy::prelude::*;
use std::collections::HashMap;

const CHUNK_SIZE: isize = 80;

pub struct GameStatePlugin;
impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameState::default())
            .add_event::<NewChunkEvent>()
            .add_event::<DespawnChunkEvent>()
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .with_system(update_chunk.label("update_chunks"))
                    .with_system(
                        handle_despawn_chunk_events
                            .label("despawn_chunks")
                            .after("update_chunks"),
                    )
                    .with_system(load_new_chunks.after("despawn_chunks")),
            );
    }
}

pub struct GameState {
    pub current_chunk: Vec2,
    pub yank_strength: f32,
    pub score: usize
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            current_chunk: Vec2::default(),
            yank_strength: 10.0,
            score: 0
        }
    }
}

#[derive(PartialEq, Component)]
pub struct Chunk {
    pub position: Vec2,
}

fn update_chunk(
    mut game_state: ResMut<GameState>,
    mut new_chunk_event_writer: EventWriter<NewChunkEvent>,
    player: Query<&Transform, (With<player::Player>, Without<bot::Bot>)>,
) {
    let player = player.single().translation;
    let current_chunk = map_to_chunk(player);

    //println!("C: {:?} P {:?}", current_chunk, player);

    if current_chunk != game_state.current_chunk {
        game_state.current_chunk = current_chunk;
        new_chunk_event_writer.send(NewChunkEvent);
    }
}

pub struct NewChunkEvent;
struct DespawnChunkEvent {
    chunk_entity: Entity,
    chunk_position: Vec2,
}

pub fn map_to_chunk(point: Vec3) -> Vec2 {
    let x = if point.x >= 0.0 {
        ((point.x as isize) + (CHUNK_SIZE / 2)) / CHUNK_SIZE
    } else {
        ((point.x as isize) - (CHUNK_SIZE / 2)) / CHUNK_SIZE
    };
    let z = if point.z >= 0.0 {
        ((point.z as isize) + (CHUNK_SIZE / 2)) / CHUNK_SIZE
    } else {
        ((point.z as isize) - (CHUNK_SIZE / 2)) / CHUNK_SIZE
    };
    Vec2::new(x as f32, z as f32)
}

fn handle_despawn_chunk_events(
    mut commands: Commands,
    mut despawn_chunk_event_reader: EventReader<DespawnChunkEvent>,
    entities: Query<(Entity, &GlobalTransform), (With<CleanupMarker>, Without<Chunk>)>,
) {
    for event in despawn_chunk_event_reader.iter() {
        commands.entity(event.chunk_entity).despawn_recursive();
        //println!("Despawning {:?}", event.chunk_position);

        for (entity, transform) in entities.iter() {
            let entity_chunk = map_to_chunk(transform.translation);
            if entity_chunk == event.chunk_position {
                //println!("despawned entity at {:?} {:?} {:?}", chunk_x, chunk_z, transform.translation);
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

fn load_new_chunks(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    assets_gltf: Res<Assets<Gltf>>,
    mut new_chunk_event_reader: EventReader<NewChunkEvent>,
    mut despawn_chunk_event_writer: EventWriter<DespawnChunkEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_state: Res<GameState>,
    chunks: Query<(Entity, &Chunk)>,
    asset_server: Res<AssetServer>,
    mut component_adder: ResMut<component_adder::ComponentAdder>,
    players: Query<&player::Player, Without<bot::Bot>>,
) {
    if new_chunk_event_reader.iter().count() > 0 {
        let x = game_state.current_chunk.x;
        let z = game_state.current_chunk.y;
        let active_chunks = vec![
            Chunk {
                position: Vec2::new(x, z),
            },
            Chunk {
                position: Vec2::new(x + 1.0, z),
            },
            Chunk {
                position: Vec2::new(x - 1.0, z),
            },
            Chunk {
                position: Vec2::new(x, z + 1.0),
            },
            Chunk {
                position: Vec2::new(x, z - 1.0),
            },
            Chunk {
                position: Vec2::new(x + 1.0, z + 1.0),
            },
            Chunk {
                position: Vec2::new(x + 1.0, z - 1.0),
            },
            Chunk {
                position: Vec2::new(x - 1.0, z + 1.0),
            },
            Chunk {
                position: Vec2::new(x - 1.0, z - 1.0),
            },
        ];

        for (entity, chunk) in chunks.iter() {
            //println!("Checking chunk {:?}", chunk.position);
            if !active_chunks.contains(chunk) {
                //println!("despawning chunk {:?}", chunk.position);

                //println!("Current {:?}", game_state.current_chunk);
                despawn_chunk_event_writer.send(DespawnChunkEvent {
                    chunk_entity: entity,
                    chunk_position: chunk.position,
                });
            }
        }

        let chunks = chunks.iter().map(|(_, c)| c).collect::<Vec<_>>();
        active_chunks
            .iter()
            .filter(|c| !chunks.contains(c))
            .for_each(|c| {
                if let Some(gltf) = assets_gltf.get(&game_assets.chunk) {
                    //println!("creating at {} {}", c.position.x , c.position.y);
                    commands
                        .spawn_bundle(TransformBundle::from_transform(Transform::from_xyz(
                            c.position.x * (CHUNK_SIZE as f32),
                            -0.5,
                            c.position.y * CHUNK_SIZE as f32,
                        )))
                        .insert(Chunk {
                            position: c.position,
                        })
                        .insert(CleanupMarker)
                        .with_children(|parent| {
                            parent.spawn_scene(gltf.scenes[0].clone());
                        });
                            
                    let min_x = (c.position.x * (CHUNK_SIZE as f32)) - (CHUNK_SIZE as f32 / 2.0);
                    let max_x = (c.position.x * (CHUNK_SIZE as f32)) + (CHUNK_SIZE as f32 / 2.0);
                    let min_z = (c.position.y * (CHUNK_SIZE as f32)) - (CHUNK_SIZE as f32 / 2.0);
                    let max_z = (c.position.y * (CHUNK_SIZE as f32)) + (CHUNK_SIZE as f32 / 2.0);
                    for _ in 0..20 {
                        let spot = get_random_spot(min_x, max_x, min_z, max_z);
                        let (target, model) = target::make_random_target();
                        commands
                            .spawn_bundle((
                                Transform::from_xyz(spot.x, 0.0, spot.y),
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
                                        parent.spawn_scene(asset_server.load(&model));
                                    });
                            })
                            .insert(CleanupMarker)
                            .insert(target);
                    }

                    if let Ok(player) = players.get_single() {
                        if player.looking_for_pets() {
                            for _ in 0..10 {
                                let spot = get_random_spot(min_x, max_x, min_z, max_z);
                                let (pickup, model) = pickup::make_random_pet();

                                commands
                                    .spawn_bundle((
                                        Transform::from_xyz(spot.x, 0.0, spot.y),
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
                                                parent.spawn_scene(asset_server.load(&model));
                                            });
                                    })
                                    .insert(CleanupMarker)
                                    .insert(pickup);
                            }
                        }
                    }

                    for _ in 0..100 {
                        let spot = get_random_spot(min_x, max_x, min_z, max_z);
                        commands
                            .spawn_bundle(PbrBundle {
                                mesh: meshes.add(Mesh::from(shape::Icosphere {
                                    radius: 0.25,
                                    subdivisions: 0,
                                })),
                                material: materials.add(Color::YELLOW.into()),
                                transform: Transform::from_xyz(spot.x, 0.0, spot.y),
                                ..Default::default()
                            })
                            .insert(CleanupMarker)
                            .insert(pickup::Pickup {
                                pickup_type: pickup::PickupType::Coin
                            });
                    }
                }
            });

        component_adder.reset();
    }
}

fn cleanup_stale_chunks() {}

pub fn get_random_spot(min_x: f32, max_x: f32, min_z: f32, max_z: f32) -> Vec2 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let x: f32 = rng.gen_range(min_x..max_x);
    let z: f32 = rng.gen_range(min_z..max_z);

    Vec2::new(x, z)
}
