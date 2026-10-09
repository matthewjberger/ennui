use std::any::{Any, TypeId};
use std::collections::HashMap;

pub(crate) type Slot = Box<dyn Any + Send + Sync>;

#[derive(Default)]
pub struct Resources {
    pub(crate) slots: HashMap<TypeId, Slot>,
    pub(crate) names: HashMap<TypeId, &'static str>,
    pub(crate) shown: HashMap<TypeId, fn(&Resources) -> Option<String>>,
}

pub fn insert<T: Send + Sync + 'static>(resources: &mut Resources, value: T) {
    resources.slots.insert(TypeId::of::<T>(), Box::new(value));
    resources
        .names
        .insert(TypeId::of::<T>(), std::any::type_name::<T>());
}

pub fn get<T: Send + Sync + 'static>(resources: &Resources) -> &T {
    resources
        .slots
        .get(&TypeId::of::<T>())
        .and_then(|slot| slot.downcast_ref())
        .unwrap_or_else(|| panic!("nothing inserted {}", std::any::type_name::<T>()))
}

pub fn get_mut<T: Send + Sync + 'static>(resources: &mut Resources) -> &mut T {
    resources
        .slots
        .get_mut(&TypeId::of::<T>())
        .and_then(|slot| slot.downcast_mut())
        .unwrap_or_else(|| panic!("nothing inserted {}", std::any::type_name::<T>()))
}

pub fn hold<T: Send + Sync + Default + 'static>(resources: &mut Resources) -> &mut T {
    resources
        .names
        .insert(TypeId::of::<T>(), std::any::type_name::<T>());
    resources
        .slots
        .entry(TypeId::of::<T>())
        .or_insert_with(|| Box::new(T::default()))
        .downcast_mut()
        .expect("a slot holds the type its key names")
}

pub fn holds(resources: &Resources, key: &TypeId) -> bool {
    resources.slots.contains_key(key)
}

pub fn show_values<T: Send + Sync + std::fmt::Debug + 'static>(resources: &mut Resources) {
    resources.shown.insert(TypeId::of::<T>(), |resources| {
        let value = resources
            .slots
            .get(&TypeId::of::<T>())?
            .downcast_ref::<T>()?;
        Some(format!("{value:?}"))
    });
}

pub fn show_with<T: Send + Sync + 'static>(
    resources: &mut Resources,
    shown: fn(&Resources) -> Option<String>,
) {
    resources.shown.insert(TypeId::of::<T>(), shown);
}
