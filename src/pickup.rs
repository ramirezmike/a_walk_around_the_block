use crate::{AppState, player, bot, game_state, audio, assets::GameAssets};
use bevy::prelude::*;

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

#[derive(Clone, Copy)]
pub enum PickupType {
    Pet,
    Coin
}

fn handle_pickup_event( 
    mut commands: Commands,
    mut pickup_event_reader: EventReader<PickupEvent>,
    mut audio: audio::GameAudio,
    game_assets: Res<GameAssets>,
) {
    for event in pickup_event_reader.iter() {
        commands.entity(event.entity).despawn_recursive();
        audio.play_sfx(&game_assets.pickup);
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
        transform.rotate(Quat::from_rotation_x(time.delta_seconds()));
    }
}
