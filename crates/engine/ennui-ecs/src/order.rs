use crate::system::{System, conflicts};
use std::any::{TypeId, type_name};
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::ops::Range;

pub struct Order {
    pub(crate) earlier: TypeId,
    pub(crate) later: TypeId,
    pub(crate) earlier_name: &'static str,
    pub(crate) later_name: &'static str,
    pub(crate) loose: bool,
}

pub fn order<Earlier: 'static, Later: 'static>(_earlier: Earlier, _later: Later) -> Order {
    Order {
        earlier: TypeId::of::<Earlier>(),
        later: TypeId::of::<Later>(),
        earlier_name: type_name::<Earlier>(),
        later_name: type_name::<Later>(),
        loose: false,
    }
}

pub fn loose_order<Earlier: 'static, Later: 'static>(earlier: Earlier, later: Later) -> Order {
    Order {
        loose: true,
        ..order(earlier, later)
    }
}

fn stages_of<S: Copy + Ord>(stages: &[(S, &[System])], key: TypeId) -> Vec<S> {
    stages
        .iter()
        .filter(|(_, systems)| systems.iter().any(|system| system.keys.contains(&key)))
        .map(|(stage, _)| *stage)
        .collect()
}

pub fn check<S: Copy + Ord>(orders: &[Order], stages: &[(S, &[System])]) {
    for order in orders {
        let earlier = stages_of(stages, order.earlier);
        let later = stages_of(stages, order.later);
        if order.loose && earlier.is_empty() {
            continue;
        }
        for (found, name) in [(&earlier, order.earlier_name), (&later, order.later_name)] {
            assert!(
                !found.is_empty(),
                "the order {} before {} names {name}, which no plugin scheduled and no step joined",
                order.earlier_name,
                order.later_name,
            );
        }
        let shared = earlier.iter().any(|stage| later.contains(stage));
        assert!(
            shared || earlier.iter().max() < later.iter().min(),
            "{} must run before {}, but it runs in a later stage",
            order.earlier_name,
            order.later_name,
        );
    }
}

fn places(systems: &[System], key: TypeId) -> Vec<usize> {
    systems
        .iter()
        .enumerate()
        .filter(|(_, system)| system.keys.contains(&key))
        .map(|(place, _)| place)
        .collect()
}

pub fn arrange(systems: &mut Vec<System>, orders: &[Order]) {
    let count = systems.len();
    let mut successors: Vec<Vec<usize>> = vec![Vec::new(); count];
    let mut incoming = vec![0usize; count];
    for order in orders {
        for earlier in places(systems, order.earlier) {
            for later in places(systems, order.later) {
                if earlier != later && !successors[earlier].contains(&later) {
                    successors[earlier].push(later);
                    incoming[later] += 1;
                }
            }
        }
    }
    let mut ready: BinaryHeap<Reverse<usize>> = (0..count)
        .filter(|place| incoming[*place] == 0)
        .map(Reverse)
        .collect();
    let mut sequence = Vec::with_capacity(count);
    while let Some(Reverse(place)) = ready.pop() {
        sequence.push(place);
        for next in successors[place].iter() {
            incoming[*next] -= 1;
            if incoming[*next] == 0 {
                ready.push(Reverse(*next));
            }
        }
    }
    assert!(
        sequence.len() == count,
        "these systems order each other in a cycle: {:?}",
        (0..count)
            .filter(|place| incoming[*place] > 0)
            .map(|place| systems[place].name)
            .collect::<Vec<_>>()
    );
    let mut taken: Vec<Option<System>> = systems.drain(..).map(Some).collect();
    systems.extend(sequence.into_iter().filter_map(|place| taken[place].take()));
}

fn ordered(earlier: &System, later: &System, orders: &[Order]) -> bool {
    orders
        .iter()
        .any(|order| earlier.keys.contains(&order.earlier) && later.keys.contains(&order.later))
}

pub fn batch(systems: &mut Vec<System>, orders: &[Order]) -> Vec<Range<usize>> {
    let mut levels: Vec<usize> = Vec::with_capacity(systems.len());
    for (place, system) in systems.iter().enumerate() {
        let level = systems[..place]
            .iter()
            .zip(levels.iter())
            .filter(|(earlier, _)| conflicts(earlier, system) || ordered(earlier, system, orders))
            .map(|(_, level)| level + 1)
            .max()
            .unwrap_or(0);
        levels.push(level);
    }
    let mut taken: Vec<(usize, System)> = levels.into_iter().zip(systems.drain(..)).collect();
    taken.sort_by_key(|(level, _)| *level);
    let mut batches: Vec<Range<usize>> = Vec::new();
    for (place, (level, system)) in taken.into_iter().enumerate() {
        match batches.len() > level {
            true => batches[level].end = place + 1,
            false => batches.push(place..place + 1),
        }
        systems.push(system);
    }
    batches
}
