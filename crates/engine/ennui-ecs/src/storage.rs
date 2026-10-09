mod bundle;
mod column;
mod joins;

pub use bundle::Bundle;
pub use joins::{Join, Stamp, Take};

use crate::entity::Entity;
use crate::mask::{self, Mask};
use column::{Column, Columns, Keyed, Table, empty_of, slice_of};
use joins::Peaks;
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};

pub trait Component: Send + Sync + 'static {}
impl<T: Send + Sync + 'static> Component for T {}

struct Kind {
    bit: Mask,
    name: &'static str,
    empty: fn() -> Column,
}

pub type Shown = fn(&Storage, Entity) -> Option<String>;

struct Need {
    key: TypeId,
    add: fn(&mut Storage, Entity),
}

pub struct Storage {
    kinds: Keyed<Kind>,
    named: Vec<TypeId>,
    columns: Keyed<Columns>,
    required: HashMap<TypeId, Vec<Need>>,
    shown: HashMap<TypeId, Shown>,
    pub(crate) counting: AtomicBool,
    counted: Mutex<Vec<(&'static str, usize)>>,
    tables: Vec<Table>,
    holders: Vec<Vec<usize>>,
    lookup: HashMap<Mask, usize>,
    places: Vec<Option<(usize, usize)>>,
    generations: Vec<u32>,
    births: Vec<u64>,
    born: u64,
    free: Vec<u32>,
    taken: AtomicUsize,
    next: AtomicU32,
    pub(crate) tick: u64,
}

pub(crate) struct Lifted {
    kinds: Vec<TypeId>,
    own: Option<Mask>,
    sets: Vec<Columns>,
}

const SPREAD_LEAST: usize = 2048;

impl Default for Storage {
    fn default() -> Self {
        Self {
            kinds: Keyed::default(),
            named: Vec::new(),
            columns: Keyed::default(),
            required: HashMap::new(),
            shown: HashMap::new(),
            counting: AtomicBool::new(false),
            counted: Mutex::new(Vec::new()),
            tables: vec![Table::default()],
            holders: vec![Vec::new(); mask::SLOTS as usize],
            lookup: HashMap::from([(mask::EMPTY, 0)]),
            places: Vec::new(),
            generations: Vec::new(),
            births: Vec::new(),
            born: 0,
            free: Vec::new(),
            taken: AtomicUsize::new(0),
            next: AtomicU32::new(0),
            tick: 1,
        }
    }
}

fn kind<T: Component>(storage: &mut Storage) -> &Kind {
    let key = TypeId::of::<T>();
    if !storage.kinds.contains_key(&key) {
        let next = storage.named.len() as u32;
        assert!(next < crate::mask::SLOTS, "too many component types");
        storage.named.push(key);
        storage.kinds.insert(
            key,
            Kind {
                bit: mask::bit(next),
                name: std::any::type_name::<T>(),
                empty: empty_of::<T>,
            },
        );
    }
    &storage.kinds[&key]
}

fn table_for(storage: &mut Storage, mask: Mask) -> usize {
    if let Some(found) = storage.lookup.get(&mask) {
        return *found;
    }
    storage.tables.push(Table {
        mask,
        ..Table::default()
    });
    let made = storage.tables.len() - 1;
    storage.lookup.insert(mask, made);
    for slot in mask::slots(&mask) {
        storage.holders[slot as usize].push(made);
    }
    made
}

fn tables_with(storage: &Storage, wanted: Mask) -> impl Iterator<Item = usize> + '_ {
    let listed = mask::slots(&wanted)
        .map(|slot| storage.holders[slot as usize].as_slice())
        .min_by_key(|tables| tables.len());
    let every = listed.is_none().then_some(0..storage.tables.len());
    listed
        .into_iter()
        .flatten()
        .copied()
        .chain(every.into_iter().flatten())
        .filter(move |index| mask::contains(&storage.tables[*index].mask, wanted))
}

fn made_column(storage: &mut Storage, kind: TypeId, table: usize) -> &mut Columns {
    let empty = storage.kinds[&kind].empty;
    let set = storage.columns.entry(kind).or_default();
    if set.len() <= table {
        set.resize_with(table + 1, || None);
    }
    if set[table].is_none() {
        set[table] = Some(empty());
    }
    set
}

fn move_row(storage: &mut Storage, entity: Entity, from: usize, row: usize, into: usize) -> usize {
    let leaving = storage.tables[from].mask;
    let arriving = storage.tables[into].mask;
    for slot in mask::slots(&leaving) {
        if !mask::contains(&arriving, mask::bit(slot)) {
            continue;
        }
        let kind = storage.named[slot as usize];
        let set = made_column(&mut *storage, kind, into);
        let [leaving, arriving] = set
            .get_disjoint_mut([from, into])
            .expect("a row moves between two different tables");
        let leaving = leaving
            .as_mut()
            .expect("a table with rows holds a column for each kind in its mask");
        let arriving = arriving.as_mut().expect("the column was just made");
        (leaving.carry)(&mut *leaving.values, row, &mut *arriving.values);
        let stamp = leaving.changed.swap_remove(row);
        arriving.changed.push(stamp);
        arriving.peak = arriving.peak.max(stamp);
    }
    vacate(storage, from, row);
    storage.tables[into].entities.push(entity);
    let landed = storage.tables[into].entities.len() - 1;
    storage.places[entity.index as usize] = Some((into, landed));
    landed
}

fn vacate(storage: &mut Storage, table: usize, row: usize) {
    storage.tables[table].entities.swap_remove(row);
    if let Some(moved) = storage.tables[table].entities.get(row).copied() {
        storage.places[moved.index as usize] = Some((table, row));
    }
}

pub fn spawn_with<B: Bundle>(storage: &mut Storage, bundle: B) -> Entity {
    let entity = match storage.free.pop() {
        Some(index) => Entity {
            index,
            generation: storage.generations[index as usize],
        },
        None => reserve(storage),
    };
    place(&mut *storage, entity);
    bundle.write(storage, entity);
    entity
}

pub(crate) fn reserve(storage: &Storage) -> Entity {
    let taken = storage.taken.fetch_add(1, Ordering::Relaxed);
    match storage.free.len().checked_sub(taken + 1) {
        Some(place) => {
            let index = storage.free[place];
            Entity {
                index,
                generation: storage.generations[index as usize],
            }
        }
        None => Entity {
            index: storage.next.fetch_add(1, Ordering::Relaxed),
            generation: 0,
        },
    }
}

pub(crate) fn settle_reserved(storage: &mut Storage) {
    let taken = std::mem::take(storage.taken.get_mut());
    let kept = storage.free.len().saturating_sub(taken);
    storage.free.truncate(kept);
}

pub(crate) fn place(storage: &mut Storage, entity: Entity) {
    let wanted = entity.index as usize + 1;
    if storage.generations.len() < wanted {
        storage.generations.resize(wanted, 0);
        storage.places.resize(wanted, None);
        storage.births.resize(wanted, 0);
    }
    storage.births[entity.index as usize] = storage.born;
    storage.born += 1;
    storage.tables[0].entities.push(entity);
    storage.places[entity.index as usize] = Some((0, storage.tables[0].entities.len() - 1));
}

pub fn born(storage: &Storage, entity: Entity) -> u64 {
    storage
        .births
        .get(entity.index as usize)
        .copied()
        .unwrap_or(u64::MAX)
}

pub fn attach<B: Bundle>(storage: &mut Storage, entity: Entity, bundle: B) {
    bundle.write(storage, entity);
}

pub fn despawn(storage: &mut Storage, entity: Entity) {
    if !is_alive(&*storage, entity) {
        return;
    }
    if let Some((table, row)) = storage.places[entity.index as usize].take() {
        let held = storage.tables[table].mask;
        for slot in mask::slots(&held) {
            let kind = storage.named[slot as usize];
            drop_cell(storage, kind, table, row);
        }
        vacate(storage, table, row);
    }
    storage.generations[entity.index as usize] += 1;
    storage.free.push(entity.index);
}

fn drop_cell(storage: &mut Storage, kind: TypeId, table: usize, row: usize) {
    if let Some(column) = storage
        .columns
        .get_mut(&kind)
        .and_then(|set| set.get_mut(table)?.as_mut())
    {
        (column.drop_row)(&mut *column.values, row);
        column.changed.swap_remove(row);
    }
}

pub fn clear(storage: &mut Storage) {
    for entity in living(storage) {
        despawn(storage, entity);
    }
}

fn add_default<T: Component + Default>(storage: &mut Storage, entity: Entity) {
    if !has::<T>(storage, entity) {
        set(storage, entity, T::default());
    }
}

pub fn require<T: Component, Needed: Component + Default>(storage: &mut Storage) {
    let needs = storage.required.entry(TypeId::of::<T>()).or_default();
    if needs.iter().all(|need| need.key != TypeId::of::<Needed>()) {
        needs.push(Need {
            key: TypeId::of::<Needed>(),
            add: add_default::<Needed>,
        });
    }
}

fn meet_the_needs<T: Component>(storage: &mut Storage, entity: Entity) {
    let adds: Vec<fn(&mut Storage, Entity)> = storage
        .required
        .get(&TypeId::of::<T>())
        .map(|needs| needs.iter().map(|need| need.add).collect())
        .unwrap_or_default();
    for add in adds {
        add(storage, entity);
    }
}

pub fn is_alive(storage: &Storage, entity: Entity) -> bool {
    storage.generations.get(entity.index as usize) == Some(&entity.generation)
}

pub fn table_of(storage: &Storage, entity: Entity) -> Option<usize> {
    Some(place_of(storage, entity)?.0)
}

fn place_of(storage: &Storage, entity: Entity) -> Option<(usize, usize)> {
    is_alive(storage, entity)
        .then(|| storage.places.get(entity.index as usize).copied().flatten())
        .flatten()
}

pub fn set<T: Component>(storage: &mut Storage, entity: Entity, value: T) {
    if !is_alive(&*storage, entity) {
        return;
    }
    let bit = kind::<T>(&mut *storage).bit;
    let Some((table, row)) = storage.places[entity.index as usize] else {
        return;
    };
    let arrived = !mask::contains(&storage.tables[table].mask, bit);
    let (table, row) = match arrived {
        false => (table, row),
        true => {
            let wanted = mask::with(storage.tables[table].mask, bit);
            let into = table_for(&mut *storage, wanted);
            (into, move_row(&mut *storage, entity, table, row, into))
        }
    };
    let tick = storage.tick;
    let column = made_column(&mut *storage, TypeId::of::<T>(), table)[table]
        .as_mut()
        .expect("the column was just made");
    let values = slice_of::<T>(&mut *column.values);
    match values.len() > row {
        true => values[row] = value,
        false => values.push(value),
    }
    match column.changed.len() > row {
        true => column.changed[row] = tick,
        false => column.changed.push(tick),
    }
    column.peak = tick;
    if arrived {
        meet_the_needs::<T>(storage, entity);
    }
}

pub fn remove<T: Component>(storage: &mut Storage, entity: Entity) {
    let key = TypeId::of::<T>();
    let Some(bit) = storage.kinds.get(&key).map(|kind| kind.bit) else {
        return;
    };
    let Some((table, row)) = place_of(storage, entity) else {
        return;
    };
    if !mask::contains(&storage.tables[table].mask, bit) {
        return;
    }
    drop_cell(storage, key, table, row);
    let wanted = mask::without(storage.tables[table].mask, bit);
    let into = table_for(&mut *storage, wanted);
    move_row(storage, entity, table, row, into);
}

pub fn get_mut<T: Component>(storage: &mut Storage, entity: Entity) -> Option<Stamp<'_, T>> {
    let (table, row) = place_of(storage, entity)?;
    let Storage { columns, tick, .. } = storage;
    let column = columns
        .get_mut(&TypeId::of::<T>())?
        .get_mut(table)?
        .as_mut()?;
    Some(joins::stamp_at(column, row, *tick))
}

pub fn tick(storage: &Storage) -> u64 {
    storage.tick
}

fn held_by<T: Component>(storage: &Storage) -> impl Iterator<Item = (usize, &Column)> + '_ {
    storage
        .columns
        .get(&TypeId::of::<T>())
        .into_iter()
        .flat_map(|set| set.iter().enumerate())
        .filter_map(|(index, column)| Some((index, column.as_ref()?)))
}

fn changed_where<T: Component>(
    storage: &Storage,
    since: u64,
    kept: impl Fn(usize) -> bool,
) -> Vec<Entity> {
    let mut out = Vec::new();
    for (index, column) in held_by::<T>(storage) {
        if column.peak <= since || !kept(index) {
            continue;
        }
        for (row, entity) in storage.tables[index].entities.iter().enumerate() {
            if column.changed[row] > since {
                out.push(*entity);
            }
        }
    }
    out
}

pub fn changed_since<T: Component>(storage: &Storage, since: u64) -> Vec<Entity> {
    changed_where::<T>(storage, since, |_| true)
}

pub fn changed_among<T: Component, With: Component>(storage: &Storage, since: u64) -> Vec<Entity> {
    let with = storage.columns.get(&TypeId::of::<With>());
    changed_where::<T>(storage, since, |index| {
        with.and_then(|set| set.get(index))
            .is_some_and(Option::is_some)
    })
}

pub fn get<T: Component>(storage: &Storage, entity: Entity) -> Option<&T> {
    let (table, row) = place_of(storage, entity)?;
    joins::values_in::<T>(
        storage.columns.get(&TypeId::of::<T>()).map(Vec::as_slice),
        table,
    )?
    .get(row)
}

pub fn has<T: Component>(storage: &Storage, entity: Entity) -> bool {
    get::<T>(storage, entity).is_some()
}

pub fn set_if_new<T: Component + PartialEq>(storage: &mut Storage, entity: Entity, value: T) {
    if get::<T>(&*storage, entity).is_some_and(|held| *held == value) {
        return;
    }
    set(&mut *storage, entity, value);
}

pub fn touched_since<T: Component>(storage: &Storage, since: u64) -> bool {
    held_by::<T>(storage).any(|(_, column)| column.peak > since)
}

pub fn count_of<T: Component>(storage: &Storage) -> usize {
    held_by::<T>(storage)
        .map(|(index, _)| storage.tables[index].entities.len())
        .sum()
}

pub fn owners<T: Component>(storage: &Storage) -> Vec<Entity> {
    held_by::<T>(storage)
        .flat_map(|(index, _)| storage.tables[index].entities.iter().copied())
        .collect()
}

pub fn query<'a, J: Join<'a> + 'a>(
    storage: &'a Storage,
) -> impl Iterator<Item = (Entity, J::Item)> + 'a {
    if storage.counting.load(Ordering::Relaxed) {
        let matched = joined_tables::<J>(storage, Some(mask::EMPTY))
            .map(|(index, _)| storage.tables[index].entities.len())
            .sum();
        note(storage, std::any::type_name::<J>(), matched);
    }
    joined_tables::<J>(storage, Some(mask::EMPTY)).flat_map(|(index, held)| {
        storage.tables[index]
            .entities
            .iter()
            .copied()
            .enumerate()
            .map(move |(row, entity)| (entity, J::row(&held, row)))
    })
}

fn columns_of(sets: &mut [Columns], table: usize) -> impl Iterator<Item = &mut Column> {
    sets.iter_mut().map(move |set| {
        set.get_mut(table)
            .and_then(Option::as_mut)
            .expect("a table with rows holds a column for each kind in its mask")
    })
}

pub(crate) fn lift<T: Take>(storage: &mut Storage) -> Lifted {
    let mut named = Vec::new();
    T::kinds(&mut named);
    let kinds: Vec<TypeId> = named.into_iter().map(|(kind, _)| kind).collect();
    let own = kinds
        .iter()
        .map(|kind| storage.kinds.get(kind).map(|held| held.bit))
        .try_fold(mask::EMPTY, |bits, held| Some(mask::with(bits, held?)));
    let sets = kinds
        .iter()
        .map(|kind| storage.columns.remove(kind).unwrap_or_default())
        .collect();
    Lifted { kinds, own, sets }
}

pub(crate) fn restore(storage: &mut Storage, lifted: Lifted) {
    for (kind, set) in lifted.kinds.into_iter().zip(lifted.sets) {
        storage.columns.insert(kind, set);
    }
}

fn settle_peaks(sets: &mut [Columns], table: usize, peaks: Peaks) {
    for (column, peak) in columns_of(sets, table).zip(peaks) {
        column.peak = peak;
    }
}

fn joined_tables<'a, J: Join<'a> + 'a>(
    storage: &'a Storage,
    own: Option<Mask>,
) -> impl Iterator<Item = (usize, J::Held)> + 'a {
    let sets = J::sets(storage);
    own.zip(J::wanted(storage))
        .map(|(own, joins)| mask::with(own, joins))
        .into_iter()
        .flat_map(move |bits| tables_with(storage, bits))
        .filter(|index| !storage.tables[*index].entities.is_empty())
        .filter_map(move |index| Some((index, J::hold(sets, index)?)))
}

pub(crate) fn lifted_each<T: Take, J>(
    storage: &Storage,
    lifted: &mut Lifted,
    mut visit: impl FnMut(Entity, T::Stamped<'_>, <J as Join<'_>>::Item),
) where
    J: for<'a> Join<'a>,
{
    let mut matched = 0;
    for (index, joined) in joined_tables::<J>(storage, lifted.own) {
        let table = &storage.tables[index];
        matched += table.entities.len();
        let peaks = {
            let mut parts = T::parts(&mut columns_of(&mut lifted.sets, index));
            for (row, entity) in table.entities.iter().copied().enumerate() {
                visit(
                    entity,
                    T::stamped(&mut parts, row, storage.tick),
                    <J as Join>::row(&joined, row),
                );
            }
            T::peaks(&parts)
        };
        settle_peaks(&mut lifted.sets, index, peaks);
    }
    note(storage, std::any::type_name::<(T, J)>(), matched);
}

fn spread<'t, T: Take, J: Join<'t>, F>(
    parts: T::Parts<'_>,
    start: usize,
    entities: &[Entity],
    joined: &J::Held,
    tick: u64,
    visit: &F,
) -> Peaks
where
    J::Held: Sync,
    F: for<'a> Fn(Entity, T::Stamped<'a>, J::Item) + Sync,
{
    if entities.len() <= SPREAD_LEAST {
        let mut parts = parts;
        for (offset, entity) in entities.iter().copied().enumerate() {
            visit(
                entity,
                T::stamped(&mut parts, offset, tick),
                J::row(joined, start + offset),
            );
        }
        return T::peaks(&parts);
    }
    let middle = entities.len() / 2;
    let (low, high) = T::split(parts, middle);
    let (mut peaks, high_peaks) = rayon::join(
        || spread::<T, J, F>(low, start, &entities[..middle], joined, tick, visit),
        || {
            spread::<T, J, F>(
                high,
                start + middle,
                &entities[middle..],
                joined,
                tick,
                visit,
            )
        },
    );
    for (held, newer) in peaks.iter_mut().zip(high_peaks) {
        *held = (*held).max(newer);
    }
    peaks
}

pub(crate) fn lifted_each_parallel<T: Take, J, F>(storage: &Storage, lifted: &mut Lifted, visit: F)
where
    J: for<'a> Join<'a>,
    for<'a> <J as Join<'a>>::Held: Sync,
    F: for<'a, 'b> Fn(Entity, T::Stamped<'a>, <J as Join<'b>>::Item) + Sync,
{
    let mut matched = 0;
    for (index, joined) in joined_tables::<J>(storage, lifted.own) {
        let table = &storage.tables[index];
        matched += table.entities.len();
        let parts = T::parts(&mut columns_of(&mut lifted.sets, index));
        let peaks = spread::<T, J, F>(parts, 0, &table.entities, &joined, storage.tick, &visit);
        settle_peaks(&mut lifted.sets, index, peaks);
    }
    note(storage, std::any::type_name::<(T, J)>(), matched);
}

pub(crate) fn lifted_row<'a, T: Take>(
    storage: &Storage,
    lifted: &'a mut Lifted,
    entity: Entity,
) -> Option<T::Stamped<'a>> {
    let (table, row) = place_of(storage, entity)?;
    let own = lifted.own?;
    if !mask::contains(&storage.tables[table].mask, own) {
        return None;
    }
    Some(T::stamped_at(
        &mut columns_of(&mut lifted.sets, table),
        row,
        storage.tick,
    ))
}

fn note(storage: &Storage, query: &'static str, matched: usize) {
    if storage.counting.load(Ordering::Relaxed)
        && let Ok(mut counted) = storage.counted.lock()
    {
        counted.push((query, matched));
    }
}

pub(crate) fn stop_counting(storage: &Storage) -> Vec<(&'static str, usize)> {
    storage.counting.store(false, Ordering::Relaxed);
    storage
        .counted
        .lock()
        .map(|mut counted| std::mem::take(&mut *counted))
        .unwrap_or_default()
}

pub fn show_values<T: Component + std::fmt::Debug>(storage: &mut Storage) {
    storage.shown.insert(TypeId::of::<T>(), |storage, entity| {
        get::<T>(storage, entity).map(|value| format!("{value:?}"))
    });
}

pub fn show_with<T: Component>(storage: &mut Storage, shown: Shown) {
    storage.shown.insert(TypeId::of::<T>(), shown);
}

pub fn living(storage: &Storage) -> Vec<Entity> {
    storage
        .tables
        .iter()
        .flat_map(|table| table.entities.iter().copied())
        .collect()
}

pub fn parts_of(storage: &Storage, entity: Entity) -> Vec<(&'static str, Option<String>)> {
    let Some((table, _)) = storage.places.get(entity.index as usize).copied().flatten() else {
        return Vec::new();
    };
    let mut parts: Vec<(&'static str, Option<String>)> = mask::slots(&storage.tables[table].mask)
        .map(|slot| storage.named[slot as usize])
        .map(|kind| {
            (
                storage.kinds[&kind].name,
                storage
                    .shown
                    .get(&kind)
                    .and_then(|show| show(storage, entity)),
            )
        })
        .collect();
    parts.sort_by_key(|(name, _)| *name);
    parts
}
