use crate::entity::Entity;
use crate::mask::Mask;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

pub(super) type Values = dyn Any + Send + Sync;

pub struct Column {
    pub(super) values: Box<Values>,
    pub(super) changed: Vec<u64>,
    pub(super) peak: u64,
    pub(super) carry: fn(&mut Values, usize, &mut Values),
    pub(super) drop_row: fn(&mut Values, usize),
}

pub(super) type Columns = Vec<Option<Column>>;

pub(super) fn empty_of<T: super::Component>() -> Column {
    Column {
        values: Box::new(Vec::<T>::new()),
        changed: Vec::new(),
        peak: 0,
        carry: carry_of::<T>,
        drop_row: drop_row_of::<T>,
    }
}

fn carry_of<T: super::Component>(from: &mut Values, row: usize, into: &mut Values) {
    let (Some(from), Some(into)) = (from.downcast_mut::<Vec<T>>(), into.downcast_mut::<Vec<T>>())
    else {
        return;
    };
    into.push(from.swap_remove(row));
}

fn drop_row_of<T: super::Component>(from: &mut Values, row: usize) {
    if let Some(from) = from.downcast_mut::<Vec<T>>() {
        from.swap_remove(row);
    }
}

pub(super) fn slice_of<T: super::Component>(values: &mut Values) -> &mut Vec<T> {
    values
        .downcast_mut::<Vec<T>>()
        .expect("the column holds this component")
}

#[derive(Default)]
pub(super) struct Plain(u64);

impl Hasher for Plain {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 = self.0.rotate_left(8) ^ u64::from(*byte);
        }
    }
    fn write_u64(&mut self, value: u64) {
        self.0 ^= value;
    }
}

pub(super) type Keyed<V> = HashMap<TypeId, V, BuildHasherDefault<Plain>>;

#[derive(Default)]
pub(super) struct Table {
    pub(super) mask: Mask,
    pub(super) entities: Vec<Entity>,
}
