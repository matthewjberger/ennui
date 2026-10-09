use super::column::{Column, slice_of};
use super::{Component, Storage};
use crate::mask::{self, Mask};
use std::any::TypeId;
use std::ops::{Deref, DerefMut};

pub(super) const MOST_TAKEN: usize = 8;

pub(super) type Peaks = [u64; MOST_TAKEN];

pub trait Take {
    type Parts<'a>: Send;
    type Stamped<'a>;
    fn kinds(out: &mut Vec<(TypeId, &'static str)>);
    fn parts<'a>(columns: &mut impl Iterator<Item = &'a mut Column>) -> Self::Parts<'a>;
    fn split<'a>(parts: Self::Parts<'a>, at: usize) -> (Self::Parts<'a>, Self::Parts<'a>);
    fn stamped<'b>(parts: &'b mut Self::Parts<'_>, row: usize, tick: u64) -> Self::Stamped<'b>;
    fn peaks(parts: &Self::Parts<'_>) -> Peaks;
    fn stamped_at<'a>(
        columns: &mut impl Iterator<Item = &'a mut Column>,
        row: usize,
        tick: u64,
    ) -> Self::Stamped<'a>;
}

pub struct Part<'a, T> {
    values: &'a mut [T],
    changed: &'a mut [u64],
    peak: u64,
}

pub struct Stamp<'a, T> {
    value: &'a mut T,
    changed: &'a mut u64,
    peak: &'a mut u64,
    tick: u64,
}

impl<T> Deref for Stamp<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.value
    }
}

impl<T> DerefMut for Stamp<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        *self.changed = self.tick;
        *self.peak = self.tick;
        self.value
    }
}

fn part_of<T: Component>(column: &mut Column) -> Part<'_, T> {
    Part {
        values: slice_of::<T>(&mut *column.values).as_mut_slice(),
        changed: column.changed.as_mut_slice(),
        peak: column.peak,
    }
}

fn split_part<T>(part: Part<'_, T>, at: usize) -> (Part<'_, T>, Part<'_, T>) {
    let (low_values, high_values) = part.values.split_at_mut(at);
    let (low_changed, high_changed) = part.changed.split_at_mut(at);
    (
        Part {
            values: low_values,
            changed: low_changed,
            peak: part.peak,
        },
        Part {
            values: high_values,
            changed: high_changed,
            peak: part.peak,
        },
    )
}

pub(super) fn stamp_at<'a, T: Component>(
    column: &'a mut Column,
    row: usize,
    tick: u64,
) -> Stamp<'a, T> {
    Stamp {
        value: &mut slice_of::<T>(&mut *column.values)[row],
        changed: &mut column.changed[row],
        peak: &mut column.peak,
        tick,
    }
}

fn stamp_of<'b, T>(part: &'b mut Part<'_, T>, row: usize, tick: u64) -> Stamp<'b, T> {
    Stamp {
        value: &mut part.values[row],
        changed: &mut part.changed[row],
        peak: &mut part.peak,
        tick,
    }
}

fn next_column<'a>(columns: &mut impl Iterator<Item = &'a mut Column>) -> &'a mut Column {
    columns
        .next()
        .expect("a table holds a column for each component it takes")
}

pub trait Join<'a> {
    type Item;
    type Held;
    type Sets: Copy;
    fn kinds(out: &mut Vec<(TypeId, &'static str)>);
    fn wanted(storage: &Storage) -> Option<Mask>;
    fn sets(storage: &'a Storage) -> Self::Sets;
    fn hold(sets: Self::Sets, table: usize) -> Option<Self::Held>;
    fn row(held: &Self::Held, row: usize) -> Self::Item;
}

impl<'a> Join<'a> for () {
    type Item = ();
    type Held = ();
    type Sets = ();
    fn kinds(_out: &mut Vec<(TypeId, &'static str)>) {}
    fn wanted(_storage: &Storage) -> Option<Mask> {
        Some(mask::EMPTY)
    }
    fn sets(_storage: &'a Storage) {}
    fn hold(_sets: (), _table: usize) -> Option<()> {
        Some(())
    }
    fn row(_held: &(), _row: usize) {}
}

pub(super) fn values_in<T: Component>(
    set: Option<&[Option<Column>]>,
    table: usize,
) -> Option<&[T]> {
    set?.get(table)?
        .as_ref()?
        .values
        .downcast_ref::<Vec<T>>()
        .map(Vec::as_slice)
}

impl<'a, T: Component> Join<'a> for &T {
    type Item = &'a T;
    type Held = &'a [T];
    type Sets = Option<&'a [Option<Column>]>;
    fn kinds(out: &mut Vec<(TypeId, &'static str)>) {
        out.push((TypeId::of::<T>(), std::any::type_name::<T>()));
    }
    fn wanted(storage: &Storage) -> Option<Mask> {
        storage.kinds.get(&TypeId::of::<T>()).map(|kind| kind.bit)
    }
    fn sets(storage: &'a Storage) -> Self::Sets {
        storage.columns.get(&TypeId::of::<T>()).map(Vec::as_slice)
    }
    fn hold(sets: Self::Sets, table: usize) -> Option<&'a [T]> {
        values_in::<T>(sets, table)
    }
    fn row(held: &&'a [T], row: usize) -> &'a T {
        &held[row]
    }
}

impl<'a, T: Component> Join<'a> for Option<&T> {
    type Item = Option<&'a T>;
    type Held = Option<&'a [T]>;
    type Sets = Option<&'a [Option<Column>]>;
    fn kinds(out: &mut Vec<(TypeId, &'static str)>) {
        out.push((TypeId::of::<T>(), std::any::type_name::<T>()));
    }
    fn wanted(_storage: &Storage) -> Option<Mask> {
        Some(mask::EMPTY)
    }
    fn sets(storage: &'a Storage) -> Self::Sets {
        storage.columns.get(&TypeId::of::<T>()).map(Vec::as_slice)
    }
    fn hold(sets: Self::Sets, table: usize) -> Option<Self::Held> {
        Some(values_in::<T>(sets, table))
    }
    fn row(held: &Option<&'a [T]>, row: usize) -> Option<&'a T> {
        held.and_then(|values| values.get(row))
    }
}

macro_rules! joins {
    ($first:ident $held:ident $(, $kind:ident $rest:ident)*) => {
        impl<'a, $first: Join<'a> $(, $kind: Join<'a>)*> Join<'a> for ($first, $($kind,)*) {
            type Item = ($first::Item, $($kind::Item,)*);
            type Held = ($first::Held, $($kind::Held,)*);
            type Sets = ($first::Sets, $($kind::Sets,)*);
            fn kinds(out: &mut Vec<(TypeId, &'static str)>) {
                $first::kinds(out);
                $($kind::kinds(out);)*
            }
            fn wanted(storage: &Storage) -> Option<Mask> {
                let bits = $first::wanted(storage)?;
                $(let bits = mask::with(bits, $kind::wanted(storage)?);)*
                Some(bits)
            }
            fn sets(storage: &'a Storage) -> Self::Sets {
                ($first::sets(storage), $($kind::sets(storage),)*)
            }
            fn hold(sets: Self::Sets, table: usize) -> Option<Self::Held> {
                let ($held, $($rest,)*) = sets;
                Some(($first::hold($held, table)?, $($kind::hold($rest, table)?,)*))
            }
            fn row(held: &Self::Held, row: usize) -> Self::Item {
                let ($held, $($rest,)*) = held;
                ($first::row($held, row), $($kind::row($rest, row),)*)
            }
        }
    };
}

macro_rules! takes {
    ($($kind:ident $held:ident $place:literal),+) => {
        impl<$($kind: Component),+> Take for ($($kind,)+) {
            type Parts<'a> = ($(Part<'a, $kind>,)+);
            type Stamped<'a> = ($(Stamp<'a, $kind>,)+);
            fn kinds(out: &mut Vec<(TypeId, &'static str)>) {
                $(out.push((TypeId::of::<$kind>(), std::any::type_name::<$kind>()));)+
            }
            fn parts<'a>(columns: &mut impl Iterator<Item = &'a mut Column>) -> Self::Parts<'a> {
                ($(part_of::<$kind>(next_column(columns)),)+)
            }
            fn split<'a>(parts: Self::Parts<'a>, at: usize) -> (Self::Parts<'a>, Self::Parts<'a>) {
                let ($($held,)+) = parts;
                let ($($held,)+) = ($(split_part($held, at),)+);
                (($($held.0,)+), ($($held.1,)+))
            }
            fn stamped<'b>(parts: &'b mut Self::Parts<'_>, row: usize, tick: u64) -> Self::Stamped<'b> {
                let ($($held,)+) = parts;
                ($(stamp_of($held, row, tick),)+)
            }
            fn peaks(parts: &Self::Parts<'_>) -> Peaks {
                let ($($held,)+) = parts;
                let mut peaks = [0; MOST_TAKEN];
                $(peaks[$place] = $held.peak;)+
                peaks
            }
            fn stamped_at<'a>(
                columns: &mut impl Iterator<Item = &'a mut Column>,
                row: usize,
                tick: u64,
            ) -> Self::Stamped<'a> {
                ($(stamp_at::<$kind>(next_column(columns), row, tick),)+)
            }
        }
    };
}

takes!(A a 0);
takes!(A a 0, B b 1);
takes!(A a 0, B b 1, C c 2);
takes!(A a 0, B b 1, C c 2, D d 3);
takes!(A a 0, B b 1, C c 2, D d 3, E e 4);
takes!(A a 0, B b 1, C c 2, D d 3, E e 4, F f 5);
takes!(A a 0, B b 1, C c 2, D d 3, E e 4, F f 5, G g 6);
takes!(A a 0, B b 1, C c 2, D d 3, E e 4, F f 5, G g 6, H h 7);

joins!(A a);
joins!(A a, B b);
joins!(A a, B b, C c);
joins!(A a, B b, C c, D d);
joins!(A a, B b, C c, D d, E e);
joins!(A a, B b, C c, D d, E e, F f);
joins!(A a, B b, C c, D d, E e, F f, G g);
joins!(A a, B b, C c, D d, E e, F f, G g, H h);
joins!(A a, B b, C c, D d, E e, F f, G g, H h, I i);
