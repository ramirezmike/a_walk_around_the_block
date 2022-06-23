use crate::{assets::GameAssets, bot, component_adder, player, AppState, CleanupMarker};
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

#[derive(Default)]
pub struct GameState {
    pub current_chunk: Vec2,
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

fn map_to_chunk(point: Vec3) -> Vec2 {
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
    game_state: Res<GameState>,
    chunks: Query<(Entity, &Chunk)>,
    mut component_adder: ResMut<component_adder::ComponentAdder>,
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
                }
            });

        component_adder.reset();
    }
}

fn cleanup_stale_chunks() {}
