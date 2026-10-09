use crate::later::apply;
use crate::resources::{Resources, Slot};
use crate::storage::{Lifted, Storage, restore, settle_reserved, stop_counting};
use crate::system::{Call, Given, Need, System, lift_for};
use crate::trace::traced;
use std::any::TypeId;
use std::collections::HashMap;
use std::ops::Range;
use std::sync::atomic::Ordering;
use web_time::Instant;

pub struct Tally {
    pub system: &'static str,
    pub queries: Vec<(String, usize)>,
}

pub fn once(system: &mut System, storage: &mut Storage, resources: &mut Resources) {
    let _traced = traced(system.name);
    (system.run)(Call::Alone(storage, resources, &mut system.queue));
    settle_reserved(storage);
    apply(&mut system.queue, storage);
}

pub fn names(systems: &[System], batches: &[Range<usize>]) -> Vec<Vec<&'static str>> {
    batches
        .iter()
        .map(|range| {
            systems[range.clone()]
                .iter()
                .map(|system| system.name)
                .collect()
        })
        .collect()
}

pub struct Cost {
    pub system: &'static str,
    pub spent: f32,
}

pub fn costs(systems: &[System]) -> Vec<Cost> {
    systems
        .iter()
        .map(|system| Cost {
            system: system.name,
            spent: system.spent,
        })
        .collect()
}

const WORTH: f32 = 0.000_05;

const BLEND: f32 = 0.125;

fn timed(spent: &mut f32, name: &'static str, run: impl FnOnce()) {
    let _traced = traced(name);
    let started = Instant::now();
    run();
    *spent += (started.elapsed().as_secs_f32() - *spent) * BLEND;
}

fn run_inline(
    systems: &mut [System],
    storage: &mut Storage,
    resources: &mut Resources,
    tallies: &mut Option<Vec<Tally>>,
) {
    for system in systems.iter_mut() {
        let name = system.name;
        if tallies.is_some() {
            storage.counting.store(true, Ordering::Relaxed);
        }
        let mut spent = system.spent;
        timed(&mut spent, name, || {
            (system.run)(Call::Alone(storage, resources, &mut system.queue))
        });
        system.spent = spent;
        if let Some(tallies) = tallies.as_mut() {
            tallies.push(Tally {
                system: name,
                queries: stop_counting(storage)
                    .into_iter()
                    .map(|(query, matched)| (String::from(query), matched))
                    .collect(),
            });
        }
    }
    settle(systems, storage);
}

fn run_parallel(systems: &mut [System], storage: &mut Storage, resources: &mut Resources) {
    let mut lifted: Vec<Vec<Option<Lifted>>> = systems
        .iter()
        .map(|system| {
            system
                .needs
                .iter()
                .map(|need| lift_for(need, storage))
                .collect()
        })
        .collect();
    let mut wanted: HashMap<TypeId, bool> = HashMap::new();
    for need in systems.iter().flat_map(|system| system.needs.iter()) {
        match need {
            Need::Resource(key, writes) => *wanted.entry(*key).or_insert(false) |= *writes,
            Need::Many(keys) => wanted.extend(keys().into_iter().map(|key| (key, true))),
            _ => {}
        }
    }
    let heaviest = systems
        .iter()
        .enumerate()
        .max_by(|(_, first), (_, second)| first.spent.total_cmp(&second.spent))
        .map(|(place, _)| place);
    {
        let mut lent = lend(resources, &wanted);
        let shared = &*storage;
        let mut calls = Vec::with_capacity(systems.len());
        for (system, lifts) in systems.iter_mut().zip(lifted.iter_mut()) {
            let System {
                run,
                needs,
                queue,
                spent,
                name,
                ..
            } = system;
            let mut queue = Some(queue);
            let givens: Vec<Given> = needs
                .iter()
                .zip(lifts.iter_mut())
                .map(|(need, lift)| match need {
                    Need::Resource(key, true) => {
                        lent.write.remove(key).map_or(Given::Nothing, Given::Write)
                    }
                    Need::Resource(key, false) => lent
                        .read
                        .get(key)
                        .copied()
                        .map_or(Given::Nothing, Given::Read),
                    Need::Many(keys) => Given::Many(
                        keys()
                            .into_iter()
                            .filter_map(|key| lent.write.remove(&key))
                            .collect(),
                    ),
                    Need::Columns(_) => lift.as_mut().map_or(Given::Nothing, Given::Columns),
                    Need::Later => queue.take().map_or(Given::Nothing, Given::Later),
                    Need::Nothing => Given::Nothing,
                })
                .collect();
            calls.push((run, *name, givens, spent));
        }
        rayon::in_place_scope(|scope| {
            let mut inline = Vec::new();
            for (place, (run, name, givens, spent)) in calls.into_iter().enumerate() {
                match *spent > WORTH && Some(place) != heaviest {
                    true => scope.spawn(move |_| {
                        timed(spent, name, || run(Call::Shared(shared, givens)));
                    }),
                    false => inline.push((run, name, givens, spent)),
                }
            }
            for (run, name, givens, spent) in inline {
                timed(spent, name, || run(Call::Shared(shared, givens)));
            }
        });
    }
    for held in lifted.into_iter().flatten().flatten() {
        restore(storage, held);
    }
    settle(systems, storage);
}

struct Lent<'a> {
    read: HashMap<TypeId, &'a Slot>,
    write: HashMap<TypeId, &'a mut Slot>,
}

fn lend<'a>(resources: &'a mut Resources, wanted: &HashMap<TypeId, bool>) -> Lent<'a> {
    let mut lent = Lent {
        read: HashMap::new(),
        write: HashMap::new(),
    };
    for (key, slot) in resources.slots.iter_mut() {
        match wanted.get(key) {
            Some(true) => {
                lent.write.insert(*key, slot);
            }
            Some(false) => {
                lent.read.insert(*key, &*slot);
            }
            None => {}
        }
    }
    lent
}

fn settle(systems: &mut [System], storage: &mut Storage) {
    storage.tick += 1;
    settle_reserved(storage);
    for system in systems.iter_mut() {
        apply(&mut system.queue, storage);
    }
}

pub fn run(
    systems: &mut [System],
    batches: &[Range<usize>],
    storage: &mut Storage,
    resources: &mut Resources,
    tallies: &mut Option<Vec<Tally>>,
    steady: bool,
) {
    for range in batches {
        storage.tick += 1;
        let members = &mut systems[range.clone()];
        let heavy = members.iter().filter(|system| system.spent > WORTH).count();
        match tallies.is_none() && heavy > 1 && !steady {
            true => run_parallel(members, storage, resources),
            false => run_inline(members, storage, resources, tallies),
        }
    }
}
