use crate::data::{Described, DescribedResource, Reflect, Value};
use crate::queries::names::short_name;
use crate::queries::text::written;
use crate::resources::{Reflected, Settings};
use ennui_ecs::entity::Entity;
use ennui_ecs::later::{Edits, change};
use ennui_ecs::resources::{Resources, get, hold, show_with as show_resource};
use ennui_ecs::storage::{
    Component, Storage, get as component_of, get_mut as component_mut, remove, set, show_with,
    touched_since,
};
use std::any::TypeId;

fn write_of<T: Component + Reflect + Default>(
    storage: &mut Storage,
    entity: Entity,
    value: &Value,
) -> bool {
    if let Some(mut held) = component_mut::<T>(storage, entity) {
        return T::apply(&mut held, value);
    }
    let mut made = T::default();
    let taken = T::apply(&mut made, value);
    set(storage, entity, made);
    taken
}

fn enter(registry: &mut Reflected, held: Described) {
    if registry.keyed.contains_key(&held.key) {
        return;
    }
    if let Some(other) = registry.named.get(held.name) {
        panic!(
            "{} and {} both reflect as {}",
            registry.components[*other].full, held.full, held.name
        );
    }
    let place = registry.components.len();
    registry.named.insert(held.name, place);
    registry.keyed.insert(held.key, place);
    registry.components.push(held);
}

pub fn component<T: Component + Reflect + Default>(edits: &mut Edits, registry: &mut Reflected) {
    change(edits, |storage| {
        show_with::<T>(storage, |storage, entity| {
            component_of::<T>(storage, entity).map(|held| written(&T::value_of(held)))
        })
    });
    let full = std::any::type_name::<T>();
    enter(
        registry,
        Described {
            name: short_name(full),
            full,
            key: TypeId::of::<T>(),
            kind: T::kind(),
            fields: T::fields(),
            about: T::about(),
            read: |storage, entity| component_of::<T>(storage, entity).map(T::value_of),
            write: Some(write_of::<T>),
            fits: |value| T::apply(&mut T::default(), value),
            made: Some(|| T::value_of(&T::default())),
            remove: |storage, entity| remove::<T>(storage, entity),
            has: |storage, entity| component_of::<T>(storage, entity).is_some(),
            touched: |storage, since| touched_since::<T>(storage, since),
        },
    );
}

pub fn resource<T: Send + Sync + Reflect + Default>(resources: &mut Resources) {
    show_resource::<T>(resources, |resources| {
        Some(written(&T::value_of(get::<T>(resources))))
    });
    let held = DescribedResource {
        name: short_name(std::any::type_name::<T>()),
        key: TypeId::of::<T>(),
        fields: T::fields(),
        about: T::about(),
        fits: |value| T::apply(&mut T::default(), value),
        made: || T::value_of(&T::default()),
    };
    hold::<Settings>(resources);
    let registry = hold::<Reflected>(resources);
    if registry.resource_named.contains_key(held.name) {
        return;
    }
    registry
        .resource_named
        .insert(held.name, registry.resources.len());
    registry.resources.push(held);
}
