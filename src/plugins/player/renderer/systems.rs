use std::collections::HashMap;
use bevy::core::Name;
use bevy::math::{EulerRot, Quat, Vec3};
use bevy::prelude::{Camera, Children, Color, Commands, Entity, Gizmos, GlobalTransform, Plane3d, Query, Transform, With, Without};
use crate::plugins::player::components::PlayerEntity;
use crate::plugins::player::input::components::PlayerInputState;
use crate::plugins::player::physics::components::PlayerPhysicsOptions;
use crate::plugins::player::renderer::components::{PlayerRendererCache, PlayerRendererCached, PlayerRendererOptions, PlayerRendererState};

/// Find entity recursively with matching name
fn find_entity(
    entity: Entity,
    query_entities: &Query<(Entity, &Name)>,
    query_children: &Query<&Children>,
    key: &String,
) -> Option<Entity> {
    let part_name = Name::new(key.clone());
    // Find Name component
    for (_, name) in query_entities.iter() {
        println!("Entity {name}")
    }

    if let Ok((entity, name)) = query_entities.get(entity) {
        if part_name == *name {
            return Some(entity);
        }
    }

    // The entity has Children component, traverse
    if let Ok(children) = query_children.get(entity) {
        for child in children {
            return if let Some(entity) = find_entity(*child, query_entities, query_children, key) {
                Some(entity)
            } else {
                None
            }
        }
    }

    return None;
}


/// Pre-cache model's part
pub(super) fn cache_model(
    mut commands: Commands,
    mut query_player: Query<(
        Entity,
        &PlayerRendererOptions,
        &mut PlayerRendererCache
    ), (
        Without<PlayerRendererCached> // Make sure that only Uncached entities are going to be cached
    )>,
    query_entities: Query<(Entity, &Name)>,
    query_children: Query<&Children>
) {
    for (
        entity,
        options,
        mut cache
    ) in query_player.iter_mut() {
        println!("Caching model parts");

        let mut to_cache = HashMap::new();
        to_cache.insert(options.head_name.clone(), &mut cache.cached_head);

        let mut entity_commands = commands.entity(entity);
        for (key, entity_ref) in to_cache {
            if let Some(found) = find_entity(entity, &query_entities, &query_children, &key) {
                *entity_ref = found;
                println!("Cached \"{key}\"")
            }
        }

        // Add marker component to mark it as cached
        entity_commands.insert(PlayerRendererCached);
    }
}

pub(super) fn renderer_input(
    mut query_player: Query<
        (
            &PlayerInputState,
            &PlayerPhysicsOptions,
            &mut PlayerRendererState,
            &Transform,
        ), With<PlayerEntity>
    >,
    query_camera: Query<
        (
            &Camera,
            &GlobalTransform
        )
    >,
    mut gizmos: Gizmos
) {
    let (camera, camera_global_transform) = query_camera.single();

    for (
        input_state,
        physics_options,
        mut renderer_state,
        transform,
    ) in query_player.iter_mut() {
        let point = camera
            .viewport_to_world(camera_global_transform, input_state.pointer)
            .map(|ray| {
                let distance = if let Some(distance) = ray.intersect_plane(transform.translation, Plane3d::default()) {
                    distance
                } else {
                    physics_options.pointer_distance_max
                };

                return ray.get_point(distance);
            });

        if let Some(point) = point {
            gizmos.sphere(point, Quat::IDENTITY, 5.0, Color::CYAN);

            let diff = point - transform.translation;

            let yaw = f32::atan2(diff.x, diff.z);
            let pitch = f32::atan2(diff.y, diff.z);

            renderer_state.target_yaw = yaw;
            renderer_state.target_pitch = pitch;
        }
    }
}

pub(super) fn renderer_update(
    mut query_player: Query<(
        &mut PlayerRendererState,
        &mut PlayerRendererCache,
        &mut Transform,
    ), With<PlayerEntity>>,
    mut query_entities: Query<&mut Transform, Without<PlayerEntity>>
) {
    for (
        mut renderer_state,
        mut renderer_cache,
        mut transform
    ) in query_player.iter_mut() {
        // TODO: Currently yaw/pitch is updated immediately, Should update it smoothly and normally as real person looks
        renderer_state.yaw = renderer_state.target_yaw;
        renderer_state.pitch = renderer_state.target_pitch;

        let root_rotation = Quat::from_euler(EulerRot::XYZ, 0.0, renderer_state.yaw, 0.0);
        transform.rotation = root_rotation;

        if let Ok(mut head_transform) = query_entities.get_mut(renderer_cache.cached_head) {
            let head_rotation = Quat::from_euler(EulerRot::XYZ, renderer_state.pitch, 0.0, 0.0);
            head_transform.rotation = head_rotation;
        }
    }
}