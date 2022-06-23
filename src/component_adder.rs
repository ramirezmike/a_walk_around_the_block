use crate::{collision, leash, CleanupMarker};
use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use uuid::Uuid;

pub struct ComponentAdderPlugin;
impl Plugin for ComponentAdderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ComponentAdder::default())
            .add_system(add_components);
    }
}

#[derive(Default)]
pub struct ComponentAdder {
    has_added: bool,
    frame_cooldown: usize,
}

impl ComponentAdder {
    pub fn reset(&mut self) {
        self.has_added = false;
        self.frame_cooldown = 0;
    }
}

fn add_components(
    mut commands: Commands,
    mut items: Query<(Entity, &Aabb, &GlobalTransform, &mut Name, &mut Visibility), With<Parent>>,
    mut component_adder: ResMut<ComponentAdder>,
) {
    if component_adder.has_added {
        return;
    }
    component_adder.frame_cooldown += 1;

    // need to wait until things are actually placed in the world
    if component_adder.frame_cooldown < 2 {
        return;
    }

    for (entity, aabb, global_transform, mut name, mut visibility) in items.iter_mut() {
        let mut change_name = false;
        if name.as_str().contains("collidable") {
            let matrix = global_transform.compute_matrix();
            commands
                .entity(entity)
                .insert(collision::Collidable {
                    aabb: collision::WorldAabb {
                        min: matrix.transform_point3(aabb.min().into()),
                        max: matrix.transform_point3(aabb.max().into()),
                    },
                })
                .insert(CleanupMarker)
                .insert(leash::PathObstacle);

            change_name = true;
        }

        if change_name {
            *name = Name::new(Uuid::new_v4().to_string());
        }
    }

    component_adder.has_added = true;
}
