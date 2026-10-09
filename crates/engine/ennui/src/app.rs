use crate::census::{Census, census};
use crate::later::within;
use crate::order::{Order, arrange, batch, check, loose_order, order};
use crate::resources::{self, Resources};
use crate::schedule::{Cost, Tally, costs, names, once};
use crate::storage::{Component, Storage};
use crate::system::{Holds, IntoSystem, Res, System};
use ennui_reflect::data::{Reflect, Settled};
use ennui_reflect::resources::{Reflected, Settings};
use ennui_reflect::systems::settle;
use std::any::TypeId;
use std::collections::HashSet;
use std::ops::Range;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Stage {
    Startup,
    Input,
    Update,
    Render,
}

pub(crate) type Runner = Box<dyn FnOnce(App)>;

pub enum Step {
    Run(Stage, System),
    Order(Order),
}

pub fn on<Marker>(stage: Stage, run: impl IntoSystem<Marker>) -> Step {
    Step::Run(stage, run.into_system())
}

pub fn grouped<Group: 'static>(_group: Group, step: Step) -> Step {
    match step {
        Step::Run(stage, mut system) => {
            system.keys.push(TypeId::of::<Group>());
            Step::Run(stage, system)
        }
        order => order,
    }
}

pub fn settling<R: Send + Sync + Reflect + 'static>() -> Step {
    grouped(Settled, on(Stage::Update, settle::<R>))
}

pub fn telling<R: Send + Sync + 'static>(
    about: &'static str,
    problems: fn(&R) -> &[String],
) -> Step {
    let mut told: HashSet<String> = HashSet::new();
    on(Stage::Update, move |held: Res<R>| {
        let fresh: Vec<&str> = problems(&held)
            .iter()
            .map(String::as_str)
            .filter(|problem| !told.contains(*problem))
            .collect();
        if fresh.is_empty() {
            return;
        }
        log::warn!("{about}: {}", fresh.join("; "));
        told.extend(fresh.into_iter().map(String::from));
    })
}

pub fn before<Earlier: 'static, Later: 'static>(earlier: Earlier, later: Later) -> Step {
    Step::Order(order(earlier, later))
}

pub fn before_if_scheduled<Earlier: 'static, Later: 'static>(
    earlier: Earlier,
    later: Later,
) -> Step {
    Step::Order(loose_order(earlier, later))
}

pub fn expect<Run: 'static>(app: &mut App, _run: Run, plugin: &'static str) {
    app.expected
        .push((TypeId::of::<Run>(), std::any::type_name::<Run>(), plugin));
}

pub fn when<T: Send + Sync + 'static, Marker>(
    stage: Stage,
    wanted: impl FnMut(&T) -> bool + Send + 'static,
    run: impl IntoSystem<Marker>,
) -> Step {
    Step::Run(stage, crate::system::gated(run, wanted))
}

pub struct Steady(pub bool);

#[derive(Default)]
pub struct App {
    pub storage: Storage,
    pub resources: Resources,
    startup: Vec<System>,
    input: Vec<System>,
    update: Vec<System>,
    render: Vec<System>,
    input_batches: Vec<Range<usize>>,
    update_batches: Vec<Range<usize>>,
    render_batches: Vec<Range<usize>>,
    orders: Vec<Order>,
    expected: Vec<(TypeId, &'static str, &'static str)>,
    pub(crate) aging: Vec<(TypeId, crate::events::Age)>,
    runner: Option<Runner>,
}

pub fn new() -> App {
    App::default()
}

pub fn insert_resource<T: Send + Sync + 'static>(app: &mut App, value: T) {
    resources::insert(&mut app.resources, value);
}

pub fn component<T: Component + Reflect + Default>(app: &mut App) {
    let registry = resources::hold::<Reflected>(&mut app.resources);
    within(&mut app.storage, |later| {
        ennui_reflect::commands::component::<T>(later, registry)
    });
}

fn take_system(app: &mut App, stage: Stage, system: System) {
    for (place, reach) in system.reach.iter().enumerate() {
        let twice = reach.holds == Holds::Resource
            && system.reach[..place]
                .iter()
                .any(|earlier| earlier.holds == Holds::Resource && earlier.key == reach.key);
        assert!(!twice, "a system names {} two times", reach.name);
    }
    match stage {
        Stage::Startup => app.startup.push(system),
        Stage::Input => app.input.push(system),
        Stage::Update => app.update.push(system),
        Stage::Render => app.render.push(system),
    }
}

pub fn schedule(app: &mut App, steps: impl IntoIterator<Item = Step>) {
    for step in steps {
        match step {
            Step::Run(stage, system) => take_system(app, stage, system),
            Step::Order(order) => app.orders.push(order),
        }
    }
}

pub fn set_runner(app: &mut App, runner: Runner) {
    if app.runner.is_some() {
        panic!("a second plugin claimed the runner");
    }
    app.runner = Some(runner);
}

pub fn start(app: &mut App) {
    check(
        &app.orders,
        &[
            (Stage::Startup, &app.startup),
            (Stage::Input, &app.input),
            (Stage::Update, &app.update),
            (Stage::Render, &app.render),
        ],
    );
    for systems in [
        &mut app.startup,
        &mut app.input,
        &mut app.update,
        &mut app.render,
    ] {
        arrange(systems, &app.orders);
    }
    for system in app.startup.iter_mut() {
        once(system, &mut app.storage, &mut app.resources);
    }
    app.startup.clear();
    app.input_batches = batch(&mut app.input, &app.orders);
    app.update_batches = batch(&mut app.update, &app.orders);
    app.render_batches = batch(&mut app.render, &app.orders);
    claim_reach(app);
    claim_settles(app);
    claim_expected(app);
}

fn claim_expected(app: &App) {
    for (key, name, plugin) in app.expected.iter() {
        let scheduled = app
            .input
            .iter()
            .chain(&app.update)
            .chain(&app.render)
            .any(|system| system.keys.contains(key));
        assert!(
            scheduled,
            "{plugin}::plugin::resources was registered, but its systems were not scheduled: {name} never runs, so the plugin does nothing"
        );
    }
}

pub fn batches(app: &App) -> Vec<Vec<&'static str>> {
    let mut listed = names(&app.input, &app.input_batches);
    listed.extend(names(&app.update, &app.update_batches));
    listed.extend(names(&app.render, &app.render_batches));
    listed
}

pub fn spent(app: &App) -> Vec<Cost> {
    let mut listed = costs(&app.input);
    listed.extend(costs(&app.update));
    listed.extend(costs(&app.render));
    listed
}

fn claim_reach(app: &App) {
    for system in app.input.iter().chain(&app.update).chain(&app.render) {
        for reach in system.reach.iter() {
            let (Holds::Resource, Some(key)) = (reach.holds, reach.key) else {
                continue;
            };
            assert!(
                resources::holds(&app.resources, &key),
                "no plugin inserted {}",
                reach.name
            );
        }
    }
}

fn claim_settles(app: &App) {
    if !resources::holds(&app.resources, &TypeId::of::<Reflected>()) {
        return;
    }
    let settings = Some(TypeId::of::<Settings>());
    for held in resources::get::<Reflected>(&app.resources).resources.iter() {
        let settled = app
            .input
            .iter()
            .chain(&app.update)
            .chain(&app.render)
            .any(|system| {
                [settings, Some(held.key)].iter().all(|key| {
                    system
                        .reach
                        .iter()
                        .any(|reach| reach.holds == Holds::Resource && reach.key == *key)
                })
            });
        assert!(
            settled,
            "scenes can set {}, but no plugin scheduled the step that settles it",
            held.name
        );
    }
}

fn frame(app: &mut App, counting: bool) -> Vec<Tally> {
    for (_, age) in app.aging.iter() {
        age(&mut app.resources);
    }
    let steady = resources::holds(&app.resources, &TypeId::of::<Steady>())
        && resources::get::<Steady>(&app.resources).0;
    let mut tallies = counting.then(Vec::new);
    for (systems, batches) in [
        (&mut app.input, &app.input_batches),
        (&mut app.update, &app.update_batches),
        (&mut app.render, &app.render_batches),
    ] {
        crate::schedule::run(
            systems,
            batches,
            &mut app.storage,
            &mut app.resources,
            &mut tallies,
            steady,
        );
    }
    tallies.unwrap_or_default()
}

pub fn tick(app: &mut App) {
    let watched = resources::holds(&app.resources, &TypeId::of::<Census>())
        && resources::get::<Census>(&app.resources).wanted;
    let tallies = frame(app, watched);
    if watched {
        let taken = census(app, tallies);
        *resources::get_mut::<Census>(&mut app.resources) = taken;
    }
}

pub fn tick_counted(app: &mut App) -> Vec<Tally> {
    frame(app, true)
}

pub fn run(app: &mut App) {
    let runner = app.runner.take().expect("no plugin claimed the runner");
    runner(std::mem::take(app));
}
