use crate::entity::Entity;
use crate::storage::{Bundle, Component, Storage, place, reserve, settle_reserved};
use crate::system::{Given, Holds, Need, Param, Reach};
use std::ops::{Deref, DerefMut};

type Deferred = Box<dyn FnOnce(&mut Storage) + Send>;

#[derive(Default)]
pub struct Queue(Vec<Deferred>);

pub struct Edits<'world> {
    queue: &'world mut Queue,
}

pub struct Later<'world> {
    edits: Edits<'world>,
    storage: &'world Storage,
}

impl<'world> Deref for Later<'world> {
    type Target = Edits<'world>;
    fn deref(&self) -> &Edits<'world> {
        &self.edits
    }
}

impl<'world> DerefMut for Later<'world> {
    fn deref_mut(&mut self) -> &mut Edits<'world> {
        &mut self.edits
    }
}

pub fn spawn<B: Bundle + Send + 'static>(later: &mut Later, bundle: B) -> Entity {
    let entity = reserve(later.storage);
    later.edits.queue.0.push(Box::new(move |storage| {
        place(&mut *storage, entity);
        bundle.write(storage, entity);
    }));
    entity
}

pub fn attach<B: Bundle + Send + 'static>(edits: &mut Edits, entity: Entity, bundle: B) {
    edits.queue.0.push(Box::new(move |storage| {
        crate::storage::attach(storage, entity, bundle)
    }));
}

pub fn set<T: Component>(edits: &mut Edits, entity: Entity, value: T) {
    edits.queue.0.push(Box::new(move |storage| {
        crate::storage::set(storage, entity, value)
    }));
}

pub fn set_if_new<T: Component + PartialEq>(edits: &mut Edits, entity: Entity, value: T) {
    edits.queue.0.push(Box::new(move |storage| {
        crate::storage::set_if_new(storage, entity, value)
    }));
}

pub fn remove<T: Component>(edits: &mut Edits, entity: Entity) {
    edits.queue.0.push(Box::new(move |storage| {
        crate::storage::remove::<T>(storage, entity)
    }));
}

pub fn despawn(edits: &mut Edits, entity: Entity) {
    edits.queue.0.push(Box::new(move |storage| {
        crate::storage::despawn(storage, entity)
    }));
}

pub fn change(edits: &mut Edits, edit: impl FnOnce(&mut Storage) + Send + 'static) {
    edits.queue.0.push(Box::new(edit));
}

pub fn within<R>(storage: &mut Storage, build: impl FnOnce(&mut Later) -> R) -> R {
    let mut queue = Queue::default();
    let made = build(&mut Later {
        edits: Edits { queue: &mut queue },
        storage: &*storage,
    });
    settle_reserved(storage);
    apply(&mut queue, storage);
    made
}

pub(crate) fn apply(queue: &mut Queue, storage: &mut Storage) {
    for change in queue.0.drain(..) {
        change(storage);
    }
}

impl Param for Later<'_> {
    type Item<'world> = Later<'world>;
    fn need() -> Need {
        Need::Later
    }
    fn reach(out: &mut Vec<Reach>) {
        out.push(Reach {
            key: None,
            name: "entity indices",
            writes: true,
            holds: Holds::Entities,
        });
    }
    fn build<'world>(given: Given<'world>, storage: &'world Storage) -> Later<'world> {
        let Given::Later(queue) = given else {
            panic!("the runner gives Later its queue");
        };
        Later {
            edits: Edits { queue },
            storage,
        }
    }
}

impl Param for Edits<'_> {
    type Item<'world> = Edits<'world>;
    fn need() -> Need {
        Need::Later
    }
    fn reach(_out: &mut Vec<Reach>) {}
    fn build<'world>(given: Given<'world>, _storage: &'world Storage) -> Edits<'world> {
        let Given::Later(queue) = given else {
            panic!("the runner gives Edits its queue");
        };
        Edits { queue }
    }
}
