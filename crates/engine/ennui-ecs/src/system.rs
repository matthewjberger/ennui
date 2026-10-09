use crate::entity::Entity;
use crate::later::Queue;
use crate::resources::{Resources, Slot};
use crate::storage::{Join, Lifted, Storage, Take};
use std::any::TypeId;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub(crate) enum Call<'world> {
    Alone(
        &'world mut Storage,
        &'world mut Resources,
        &'world mut Queue,
    ),
    Shared(&'world Storage, Vec<Given<'world>>),
}

pub(crate) type ParamRun = Box<dyn for<'world> FnMut(Call<'world>) + Send>;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Holds {
    Resource,
    Component,
    Everything,
    Entities,
}

pub struct Reach {
    pub key: Option<TypeId>,
    pub name: &'static str,
    pub writes: bool,
    pub holds: Holds,
}

#[derive(Clone, Copy)]
pub(crate) enum Need {
    Nothing,
    Resource(TypeId, bool),
    Many(fn() -> Vec<TypeId>),
    Columns(fn(&mut Storage) -> Lifted),
    Later,
}

pub(crate) enum Given<'world> {
    Nothing,
    Read(&'world Slot),
    Write(&'world mut Slot),
    Many(Vec<&'world mut Slot>),
    Columns(&'world mut Lifted),
    Later(&'world mut Queue),
}

pub struct System {
    pub(crate) run: ParamRun,
    pub(crate) needs: Vec<Need>,
    pub(crate) queue: Queue,
    pub(crate) spent: f32,
    pub name: &'static str,
    pub keys: Vec<TypeId>,
    pub reach: Vec<Reach>,
}

fn made(run: ParamRun, needs: Vec<Need>, reach: Vec<Reach>) -> System {
    System {
        run,
        needs,
        queue: Queue::default(),
        spent: 0.0,
        name: "unnamed",
        keys: Vec::new(),
        reach,
    }
}

pub fn named(mut system: System, name: &'static str) -> System {
    system.name = name;
    system
}

pub(crate) fn keyed(mut system: System, key: TypeId) -> System {
    system.keys.push(key);
    system
}

pub(crate) fn conflicts(first: &System, second: &System) -> bool {
    first.reach.iter().any(|mine| {
        second
            .reach
            .iter()
            .any(|theirs| overlaps(mine, theirs) || overlaps(theirs, mine))
    })
}

fn overlaps(mine: &Reach, theirs: &Reach) -> bool {
    match (mine.holds, theirs.holds) {
        (Holds::Everything, Holds::Component) => theirs.writes,
        (held, other) => held == other && mine.key == theirs.key && (mine.writes || theirs.writes),
    }
}

fn seen<T: Send + Sync + 'static>(slot: Option<&Slot>) -> &T {
    slot.and_then(|slot| slot.downcast_ref())
        .unwrap_or_else(|| panic!("no plugin inserted {}", std::any::type_name::<T>()))
}

pub fn gated<T: Send + Sync + 'static, Marker>(
    held: impl IntoSystem<Marker>,
    mut wanted: impl FnMut(&T) -> bool + Send + 'static,
) -> System {
    let inner = held.into_system();
    let key = TypeId::of::<T>();
    let mut reach = inner.reach;
    let mut needs = inner.needs;
    let named_at = needs
        .iter()
        .position(|need| matches!(need, Need::Resource(held, _) if *held == key));
    if named_at.is_none() {
        resource_reach::<T>(&mut reach, false);
        needs.push(Need::Resource(key, false));
    }
    let mut run = inner.run;
    let run: ParamRun = Box::new(move |call| match call {
        Call::Alone(storage, resources, queue) => {
            if wanted(crate::resources::get::<T>(&*resources)) {
                run(Call::Alone(storage, resources, queue));
            }
        }
        Call::Shared(storage, mut givens) => {
            let extra = match named_at {
                Some(_) => None,
                None => givens.pop(),
            };
            let slot = match named_at.map_or(extra.as_ref(), |place| givens.get(place)) {
                Some(Given::Read(slot)) => Some(*slot),
                Some(Given::Write(slot)) => Some(&**slot),
                _ => None,
            };
            let open = wanted(seen::<T>(slot));
            if open {
                run(Call::Shared(storage, givens));
            }
        }
    });
    let mut made = named(made(run, needs, reach), inner.name);
    made.keys = inner.keys;
    made
}

pub(crate) trait Param {
    type Item<'world>;
    fn need() -> Need {
        Need::Nothing
    }
    fn reach(out: &mut Vec<Reach>);
    fn build<'world>(given: Given<'world>, storage: &'world Storage) -> Self::Item<'world>;
}

pub struct Res<'world, T>(&'world T);

impl<T> Deref for Res<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0
    }
}

pub struct ResMut<'world, T>(&'world mut T);

impl<T> Deref for ResMut<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0
    }
}

impl<T> DerefMut for ResMut<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0
    }
}

fn resource_reach<T: 'static>(out: &mut Vec<Reach>, writes: bool) {
    out.push(Reach {
        key: Some(TypeId::of::<T>()),
        name: std::any::type_name::<T>(),
        writes,
        holds: Holds::Resource,
    });
}

impl<T: Send + Sync + 'static> Param for Res<'_, T> {
    type Item<'world> = Res<'world, T>;
    fn need() -> Need {
        Need::Resource(TypeId::of::<T>(), false)
    }
    fn reach(out: &mut Vec<Reach>) {
        resource_reach::<T>(out, false);
    }
    fn build<'world>(given: Given<'world>, _storage: &'world Storage) -> Res<'world, T> {
        let slot: Option<&'world Slot> = match given {
            Given::Read(slot) => Some(slot),
            Given::Write(slot) => Some(slot),
            _ => None,
        };
        Res(seen(slot))
    }
}

impl<T: Send + Sync + 'static> Param for ResMut<'_, T> {
    type Item<'world> = ResMut<'world, T>;
    fn need() -> Need {
        Need::Resource(TypeId::of::<T>(), true)
    }
    fn reach(out: &mut Vec<Reach>) {
        resource_reach::<T>(out, true);
    }
    fn build<'world>(given: Given<'world>, _storage: &'world Storage) -> ResMut<'world, T> {
        let slot = match given {
            Given::Write(slot) => Some(slot),
            _ => None,
        };
        ResMut(
            slot.and_then(|slot| slot.downcast_mut())
                .unwrap_or_else(|| panic!("no plugin inserted {}", std::any::type_name::<T>())),
        )
    }
}

macro_rules! bundled_resources {
    ($($held:ident),+) => {
        impl<$($held: Send + Sync + 'static),+> Param for ($(ResMut<'_, $held>,)+) {
            type Item<'world> = ($(ResMut<'world, $held>,)+);
            fn need() -> Need {
                Need::Many(|| vec![$(TypeId::of::<$held>()),+])
            }
            fn reach(out: &mut Vec<Reach>) {
                $(resource_reach::<$held>(out, true);)+
            }
            fn build<'world>(given: Given<'world>, _storage: &'world Storage) -> Self::Item<'world> {
                let mut slots = match given {
                    Given::Many(slots) => slots,
                    _ => Vec::new(),
                }
                .into_iter();
                ($(ResMut(
                    slots
                        .next()
                        .and_then(|slot| slot.downcast_mut())
                        .unwrap_or_else(|| {
                            panic!("no plugin inserted {}", std::any::type_name::<$held>())
                        }),
                ),)+)
            }
        }
    };
}

bundled_resources!(A, B);
bundled_resources!(A, B, C);
bundled_resources!(A, B, C, D);
bundled_resources!(A, B, C, D, E);

pub struct View<'world, J>(&'world Storage, PhantomData<J>);

pub struct Glance<'world>(&'world Storage);

impl Deref for Glance<'_> {
    type Target = Storage;
    fn deref(&self) -> &Storage {
        self.0
    }
}

impl Param for Glance<'_> {
    type Item<'world> = Glance<'world>;
    fn reach(out: &mut Vec<Reach>) {
        out.push(Reach {
            key: None,
            name: "Glance",
            writes: false,
            holds: Holds::Everything,
        });
    }
    fn build<'world>(_given: Given<'world>, storage: &'world Storage) -> Glance<'world> {
        Glance(storage)
    }
}

pub struct Peek<'world, T>(&'world Storage, PhantomData<T>);

pub struct Mut<'world, T, J = ()> {
    lifted: &'world mut Lifted,
    storage: &'world Storage,
    kinds: PhantomData<(T, J)>,
}

pub fn peek<'a, T: crate::storage::Component>(
    held: &'a Peek<'a, T>,
    entity: Entity,
) -> Option<&'a T> {
    crate::storage::get::<T>(held.0, entity)
}

pub fn changed<T: crate::storage::Component>(held: &Peek<'_, T>, since: u64) -> Vec<Entity> {
    crate::storage::changed_since::<T>(held.0, since)
}

pub fn touched<T: crate::storage::Component>(held: &Peek<'_, T>, since: u64) -> bool {
    crate::storage::touched_since::<T>(held.0, since)
}

pub fn counted<T: crate::storage::Component>(held: &Peek<'_, T>) -> usize {
    crate::storage::count_of::<T>(held.0)
}

pub fn holders<T: crate::storage::Component>(held: &Peek<'_, T>) -> Vec<Entity> {
    crate::storage::owners::<T>(held.0)
}

pub fn alive<T: crate::storage::Component>(held: &Peek<'_, T>, entity: Entity) -> bool {
    crate::storage::is_alive(held.0, entity)
}

pub fn ticked<T: crate::storage::Component>(held: &Peek<'_, T>) -> u64 {
    crate::storage::tick(held.0)
}

pub fn each_mut<T: Take, J>(
    held: &mut Mut<'_, T, J>,
    visit: impl FnMut(Entity, T::Stamped<'_>, <J as Join<'_>>::Item),
) where
    J: for<'a> Join<'a>,
{
    crate::storage::lifted_each::<T, J>(held.storage, held.lifted, visit);
}

pub fn each_mut_parallel<T: Take, J, F>(held: &mut Mut<'_, T, J>, visit: F)
where
    J: for<'a> Join<'a>,
    for<'a> <J as Join<'a>>::Held: Sync,
    F: for<'a, 'b> Fn(Entity, T::Stamped<'a>, <J as Join<'b>>::Item) + Sync,
{
    crate::storage::lifted_each_parallel::<T, J, F>(held.storage, held.lifted, visit);
}

pub fn peek_mut<'a, T: Take, J>(
    held: &'a mut Mut<'_, T, J>,
    entity: Entity,
) -> Option<T::Stamped<'a>> {
    crate::storage::lifted_row::<T>(held.storage, held.lifted, entity)
}

impl<T: crate::storage::Component> Param for Peek<'_, T> {
    type Item<'world> = Peek<'world, T>;
    fn reach(out: &mut Vec<Reach>) {
        <View<'_, &T> as Param>::reach(out);
    }
    fn build<'world>(_given: Given<'world>, storage: &'world Storage) -> Peek<'world, T> {
        Peek(storage, PhantomData)
    }
}

pub fn each<'a, J: Join<'a> + 'a>(
    view: &'a View<'a, J>,
) -> impl Iterator<Item = (Entity, J::Item)> + 'a {
    crate::storage::query::<J>(view.0)
}

fn component_reach(out: &mut Vec<Reach>, kinds: Vec<(TypeId, &'static str)>, writes: bool) {
    for (key, name) in kinds {
        out.push(Reach {
            key: Some(key),
            name,
            writes,
            holds: Holds::Component,
        });
    }
}

impl<J> Param for View<'_, J>
where
    J: for<'a> Join<'a>,
{
    type Item<'world> = View<'world, J>;
    fn reach(out: &mut Vec<Reach>) {
        let mut kinds = Vec::new();
        <J as Join>::kinds(&mut kinds);
        component_reach(out, kinds, false);
    }
    fn build<'world>(_given: Given<'world>, storage: &'world Storage) -> View<'world, J> {
        View(storage, PhantomData)
    }
}

impl<T: Take + 'static, J> Param for Mut<'_, T, J>
where
    J: for<'a> Join<'a> + 'static,
{
    type Item<'world> = Mut<'world, T, J>;
    fn need() -> Need {
        Need::Columns(crate::storage::lift::<T>)
    }
    fn reach(out: &mut Vec<Reach>) {
        let mut written = Vec::new();
        T::kinds(&mut written);
        component_reach(out, written, true);
        <View<'_, J> as Param>::reach(out);
    }
    fn build<'world>(given: Given<'world>, storage: &'world Storage) -> Mut<'world, T, J> {
        let Given::Columns(lifted) = given else {
            panic!("the runner lifts the columns that Mut writes");
        };
        Mut {
            lifted,
            storage,
            kinds: PhantomData,
        }
    }
}

#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot run as a system",
    label = "not a system",
    note = "a system takes 1 to 12 parameters, and each one is a Res, ResMut, View, Mut, Peek, Glance, Later or Edits, or a tuple of 2 to 5 ResMut",
    note = "a system that needs more than 12 parameters does more than one job: split it into two systems"
)]
pub trait IntoSystem<Marker> {
    fn into_system(self) -> System;
}

pub struct Ready;

impl IntoSystem<Ready> for System {
    fn into_system(self) -> System {
        self
    }
}

struct Blank<const POSITION: usize>;

fn key_of(need: Need, blank: TypeId) -> TypeId {
    match need {
        Need::Resource(key, _) => key,
        _ => blank,
    }
}

pub(crate) fn lift_for(need: &Need, storage: &mut Storage) -> Option<Lifted> {
    match need {
        Need::Columns(lift) => Some(lift(storage)),
        _ => None,
    }
}

pub(crate) type Pulled = Vec<(TypeId, Slot)>;

pub(crate) fn pull_for(need: &Need, resources: &mut Resources) -> Pulled {
    let Need::Many(keys) = need else {
        return Vec::new();
    };
    keys()
        .into_iter()
        .filter_map(|key| resources.slots.remove(&key).map(|slot| (key, slot)))
        .collect()
}

fn alone_givens<'world, const COUNT: usize>(
    needs: &[Need],
    slots: [Option<&'world mut Slot>; COUNT],
    (pulled, lifted): (
        &'world mut [Pulled; COUNT],
        &'world mut [Option<Lifted>; COUNT],
    ),
    queue: &'world mut Queue,
) -> [Given<'world>; COUNT] {
    let mut queue = Some(queue);
    let mut slots = slots.into_iter();
    let mut pulls = pulled.iter_mut();
    let mut lifts = lifted.iter_mut();
    std::array::from_fn(|place| {
        let slot = slots.next().flatten();
        let pull = pulls.next();
        let lift = lifts.next().and_then(Option::as_mut);
        match needs[place] {
            Need::Resource(..) => slot.map_or(Given::Nothing, Given::Write),
            Need::Many(_) => pull.map_or(Given::Nothing, |held| {
                Given::Many(held.iter_mut().map(|(_, slot)| slot).collect())
            }),
            Need::Columns(_) => lift.map_or(Given::Nothing, Given::Columns),
            Need::Later => queue.take().map_or(Given::Nothing, Given::Later),
            Need::Nothing => Given::Nothing,
        }
    })
}

fn checked(needs: &[Need], reach: &[Reach], name: &'static str) {
    let later = needs
        .iter()
        .filter(|need| matches!(need, Need::Later))
        .count();
    assert!(
        later <= 1,
        "{name} takes more than one Later or Edits queue"
    );
    let glances = reach.iter().any(|held| held.holds == Holds::Everything);
    let lifts = needs.iter().any(|need| matches!(need, Need::Columns(_)));
    assert!(
        !(glances && lifts),
        "{name} takes Glance and Mut, but Glance cannot see the columns that Mut lifts"
    );
    for (place, mine) in reach.iter().enumerate() {
        let twice = mine.holds == Holds::Component
            && reach[..place].iter().any(|earlier| {
                earlier.holds == Holds::Component
                    && earlier.key == mine.key
                    && (earlier.writes || mine.writes)
            });
        assert!(
            !twice,
            "{name} reads and writes {} in two parameters",
            mine.name
        );
    }
}

macro_rules! parameter_systems {
    ($($param:ident $position:literal),+) => {
        impl<Func, $($param: Param),+> IntoSystem<fn($($param),+)> for Func
        where
            Func: FnMut($($param),+) + FnMut($($param::Item<'_>),+) + Send + 'static,
        {
            fn into_system(mut self) -> System {
                let name = std::any::type_name::<Func>();
                let needs = vec![$($param::need()),+];
                let mut reach = Vec::new();
                $($param::reach(&mut reach);)+
                checked(&needs, &reach, name);
                let kept = needs.clone();
                let run: ParamRun = Box::new(move |call: Call<'_>| match call {
                    Call::Alone(storage, resources, queue) => {
                        let mut pulled = [$(pull_for(&kept[$position], resources)),+];
                        let keys = [$(key_of(kept[$position], TypeId::of::<Blank<$position>>())),+];
                        let mut lifted = [$(lift_for(&kept[$position], storage)),+];
                        {
                            let slots = resources.slots.get_disjoint_mut(keys.each_ref());
                            let mut givens =
                                alone_givens(&kept, slots, (&mut pulled, &mut lifted), queue)
                                    .into_iter();
                            self($($param::build(
                                givens.next().expect("one gift for each parameter"),
                                &*storage,
                            )),+);
                        }
                        for held in lifted.into_iter().flatten() {
                            crate::storage::restore(storage, held);
                        }
                        for (key, slot) in pulled.into_iter().flatten() {
                            resources.slots.insert(key, slot);
                        }
                    }
                    Call::Shared(storage, givens) => {
                        let mut givens = givens.into_iter();
                        self($($param::build(
                            givens.next().expect("one gift for each parameter"),
                            storage,
                        )),+);
                    }
                });
                keyed(named(made(run, needs, reach), name), TypeId::of::<Func>())
            }
        }
    };
}

parameter_systems!(A 0);
parameter_systems!(A 0, B 1);
parameter_systems!(A 0, B 1, C 2);
parameter_systems!(A 0, B 1, C 2, D 3);
parameter_systems!(A 0, B 1, C 2, D 3, E 4);
parameter_systems!(A 0, B 1, C 2, D 3, E 4, F 5);
parameter_systems!(A 0, B 1, C 2, D 3, E 4, F 5, G 6);
parameter_systems!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7);
parameter_systems!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8);
parameter_systems!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9);
parameter_systems!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10);
parameter_systems!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11);
parameter_systems!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12);
parameter_systems!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12, N 13);
parameter_systems!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12, N 13, O 14);
parameter_systems!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12, N 13, O 14, P 15);
parameter_systems!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12, N 13, O 14, P 15, Q 16);
parameter_systems!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12, N 13, O 14, P 15, Q 16, R 17);
parameter_systems!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12, N 13, O 14, P 15, Q 16, R 17, S 18);
parameter_systems!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12, N 13, O 14, P 15, Q 16, R 17, S 18, T 19);
