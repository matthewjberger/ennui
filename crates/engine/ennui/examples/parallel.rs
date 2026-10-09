use ennui::app::{self, App, Step, batches, before, on, schedule, start, tick};
use ennui::census::short;
use ennui::later::{despawn, spawn};
use ennui::prelude::{Later, Mut, ResMut, Stage, View, each, each_mut, each_mut_parallel};
use ennui::storage::{query, spawn_with};
use std::time::Instant;

const ENTITIES: usize = 200_000;
const TICKS: u32 = 120;
const REPLACED_EACH_TICK: usize = 100;
const SPREAD_EVERY: usize = 97;

#[derive(Default, Clone, Copy)]
struct Phase(f32);
#[derive(Default, Clone, Copy)]
struct Position(f32);
#[derive(Default, Clone, Copy)]
struct Velocity(f32);
#[derive(Default, Clone, Copy)]
struct Heat(f32);
#[derive(Default, Clone, Copy)]
struct Charge(f32);
#[derive(Default, Clone, Copy)]
struct Wear(f32);
#[derive(Default, Clone, Copy)]
struct Spin(f32);

#[derive(Default)]
struct Replaced(usize);

#[derive(Default)]
struct Cooled(usize);

fn churn(seed: f32) -> f32 {
    (0..6).fold(seed, |held, step| {
        (held * 1.000_1 + step as f32).sin().abs()
    })
}

fn integrate(mut rows: Mut<(Position,), (&Phase,)>) {
    each_mut(&mut rows, |_, (mut position,), (phase,)| {
        position.0 = churn(position.0 + phase.0);
    });
}

fn drift(mut rows: Mut<(Velocity,), (&Phase,)>) {
    each_mut(&mut rows, |_, (mut velocity,), (phase,)| {
        velocity.0 = churn(velocity.0 * 0.99 + phase.0);
    });
}

fn warm(mut rows: Mut<(Heat,), (&Phase,)>) {
    each_mut(&mut rows, |_, (mut heat,), (phase,)| {
        heat.0 = churn(heat.0 + phase.0.sqrt());
    });
}

fn charge(mut rows: Mut<(Charge,), (&Phase,)>) {
    each_mut(&mut rows, |_, (mut charge,), (phase,)| {
        charge.0 = churn(charge.0 - phase.0);
    });
}

fn abrade(mut rows: Mut<(Wear,), (&Phase,)>) {
    each_mut(&mut rows, |_, (mut wear,), (phase,)| {
        wear.0 = churn(wear.0 + phase.0 * 0.5);
    });
}

fn rotate(mut rows: Mut<(Spin,), (&Phase,)>) {
    each_mut(&mut rows, |_, (mut spin,), (phase,)| {
        spin.0 = churn(spin.0 + phase.0 * 2.0);
    });
}

fn replace_the_hot(hot: View<(&Heat, &Phase)>, mut later: Later, mut replaced: ResMut<Replaced>) {
    let hottest = each(&hot)
        .filter(|(_, (heat, _))| heat.0 > 0.5)
        .take(REPLACED_EACH_TICK);
    for (entity, (_, phase)) in hottest {
        despawn(&mut later, entity);
        spawn(&mut later, parts(phase.0));
        replaced.0 += 1;
    }
}

fn replace_the_cold(cold: View<(&Heat, &Phase)>, mut later: Later, mut cooled: ResMut<Cooled>) {
    for (place, (entity, (heat, phase))) in each(&cold).enumerate() {
        if place % SPREAD_EVERY == 0 {
            despawn(&mut later, entity);
            spawn(&mut later, parts(churn(heat.0 + phase.0)));
            cooled.0 += 1;
        }
    }
}

fn replace_the_warm(warm: View<(&Heat, &Phase)>, mut later: Later, mut replaced: ResMut<Replaced>) {
    for (place, (entity, (heat, phase))) in each(&warm).enumerate() {
        if place % SPREAD_EVERY == 1 {
            despawn(&mut later, entity);
            spawn(&mut later, parts(churn(heat.0 * phase.0)));
            replaced.0 += 1;
        }
    }
}

type Parts = (Phase, Position, Velocity, Heat, Charge, Wear, Spin);

fn parts(phase: f32) -> Parts {
    (
        Phase(phase),
        Position::default(),
        Velocity::default(),
        Heat::default(),
        Charge::default(),
        Wear::default(),
        Spin::default(),
    )
}

fn writers() -> Vec<Step> {
    vec![
        on(Stage::Update, integrate),
        on(Stage::Update, drift),
        on(Stage::Update, warm),
        on(Stage::Update, charge),
        on(Stage::Update, abrade),
        on(Stage::Update, rotate),
        on(Stage::Update, replace_the_hot),
    ]
}

fn chain() -> Vec<Step> {
    vec![
        before(integrate, drift),
        before(drift, warm),
        before(warm, charge),
        before(charge, abrade),
        before(abrade, rotate),
    ]
}

fn settle(mut rows: Mut<(Position,), (&Phase,)>) {
    each_mut(&mut rows, |_, (mut position,), (phase,)| {
        position.0 = churn(churn(churn(position.0 + phase.0)));
    });
}

fn settle_in_parallel(mut rows: Mut<(Position,), (&Phase,)>) {
    each_mut_parallel(&mut rows, |_, (mut position,), (phase,)| {
        position.0 = churn(churn(churn(position.0 + phase.0)));
    });
}

fn made(steps: Vec<Step>) -> App {
    let mut app = app::new();
    app::insert_resource(&mut app, Replaced::default());
    app::insert_resource(&mut app, Cooled::default());
    for index in 0..ENTITIES {
        spawn_with(&mut app.storage, parts(index as f32 / ENTITIES as f32));
    }
    schedule(&mut app, steps);
    start(&mut app);
    app
}

fn measured(label: &str, steps: Vec<Step>) -> f64 {
    let mut app = made(steps);
    println!("{label}");
    for batch in batches(&app) {
        let names: Vec<String> = batch.into_iter().map(short).collect();
        println!("  [{}]", names.join(", "));
    }
    for _ in 0..10 {
        tick(&mut app);
    }
    let started = Instant::now();
    for _ in 0..TICKS {
        tick(&mut app);
    }
    let each_tick = started.elapsed().as_secs_f64() * 1000.0 / TICKS as f64;
    let replaced = ennui::resources::get::<Replaced>(&app.resources).0;
    match replaced {
        0 => println!("  {each_tick:.2} ms per tick\n"),
        _ => println!("  {each_tick:.2} ms per tick, {replaced} entities replaced through Later\n"),
    }
    each_tick
}

fn spawned_digest() -> (u64, u32) {
    let mut app = made(vec![
        on(Stage::Update, warm),
        on(Stage::Update, replace_the_warm),
        on(Stage::Update, replace_the_cold),
    ]);
    for _ in 0..TICKS {
        tick(&mut app);
    }
    query::<(&Phase,)>(&app.storage).fold((0u64, 0u32), |(digest, highest), (entity, (phase,))| {
        let row = (u64::from(entity.index) << 32 | u64::from(entity.generation))
            ^ u64::from(phase.0.to_bits());
        (
            digest.rotate_left(5) ^ row.wrapping_mul(0x9e37_79b9_7f4a_7c15),
            highest.max(entity.index),
        )
    })
}

fn main() {
    println!(
        "{ENTITIES} entities, {TICKS} ticks, {} threads\n",
        threads()
    );
    let mut steps = writers();
    steps.extend(chain());
    let serial = measured(
        "six writers chained with before (the old serial order)",
        steps,
    );
    let parallel = measured(
        "six writers free (the planner makes the batches)",
        writers(),
    );
    println!("batches: {:.2}x faster\n", serial / parallel);
    let alone = measured("one system, each_mut", vec![on(Stage::Update, settle)]);
    let divided = measured(
        "one system, each_mut_parallel",
        vec![on(Stage::Update, settle_in_parallel)],
    );
    println!(
        "rows divided between threads: {:.2}x faster
",
        alone / divided
    );
    let (digest, highest) = spawned_digest();
    println!(
        "two Later spawners side by side: entity digest {digest:016x}, highest index {highest}"
    );
}

fn threads() -> usize {
    std::thread::available_parallelism().map_or(1, |count| count.get())
}
