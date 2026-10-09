use crate::resources::{self, Resources};
use std::any::TypeId;

const EVENT_CAP: usize = 262_144;

pub type Age = fn(&mut Resources);

pub struct Events<T> {
    held: Vec<T>,
    base: u64,
    marked: u64,
}

impl<T> Default for Events<T> {
    fn default() -> Self {
        Self {
            held: Vec::new(),
            base: 0,
            marked: 0,
        }
    }
}

pub fn send<T>(events: &mut Events<T>, event: T) {
    if events.held.len() >= EVENT_CAP {
        let half = events.held.len() / 2;
        events.held.drain(..half);
        events.base += half as u64;
        events.marked = events.marked.max(events.base);
    }
    events.held.push(event);
}

pub fn read<T>(events: &Events<T>) -> &[T] {
    &events.held[..events
        .marked
        .saturating_sub(events.base)
        .min(events.held.len() as u64) as usize]
}

pub fn consume<'a, T>(events: &'a Events<T>, cursor: &mut u64) -> &'a [T] {
    let found = &events.held[cursor
        .saturating_sub(events.base)
        .min(events.held.len() as u64) as usize..];
    *cursor = events.base + events.held.len() as u64;
    found
}

pub(crate) fn advance<T>(events: &mut Events<T>) {
    let count = events
        .marked
        .saturating_sub(events.base)
        .min(events.held.len() as u64) as usize;
    events.held.drain(..count);
    events.base += count as u64;
    events.marked = events.base + events.held.len() as u64;
}

pub fn add<T: Send + Sync + 'static>(resources: &mut Resources, aging: &mut Vec<(TypeId, Age)>) {
    let key = TypeId::of::<Events<T>>();
    if aging.iter().any(|(held, _)| *held == key) {
        return;
    }
    resources::insert(resources, Events::<T>::default());
    aging.push((key, |resources| {
        advance(resources::get_mut::<Events<T>>(resources))
    }));
}
