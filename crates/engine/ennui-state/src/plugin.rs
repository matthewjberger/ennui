use crate::data::{States, Transition};
use crate::resources::State;
use crate::systems::transition;
use ennui::app::grouped;
use ennui::app::insert_resource;
use ennui::prelude::{App, Stage, Step, on};
use ennui::system::{IntoSystem, gated};

pub fn resources<S: States>(app: &mut App, first: S) {
    insert_resource(&mut *app, State::new(first));
}

pub fn systems<S: States>() -> Vec<Step> {
    vec![grouped(
        Transition,
        on(Stage::Update, transition::apply::<S>),
    )]
}

fn on_turned<S: States, Marker>(
    system: impl IntoSystem<Marker>,
    wanted: impl Fn(&State<S>) -> bool + Send + 'static,
) -> Step {
    let mut seen = 0;
    on(
        Stage::Update,
        gated(system, move |held: &State<S>| {
            let turned = seen != held.turn;
            seen = held.turn;
            turned && wanted(held)
        }),
    )
}

pub fn on_enter<S: States, Marker>(state: S, system: impl IntoSystem<Marker>) -> Step {
    on_turned(system, move |held: &State<S>| held.current == state)
}

pub fn on_turn<S: States, Marker>(system: impl IntoSystem<Marker>) -> Step {
    on_turned(system, |_: &State<S>| true)
}

pub fn on_exit<S: States, Marker>(state: S, system: impl IntoSystem<Marker>) -> Step {
    on_turned(system, move |held: &State<S>| held.from == Some(state))
}

pub fn while_in<S: States, Marker>(
    stage: Stage,
    state: S,
    system: impl IntoSystem<Marker>,
) -> Step {
    on(
        stage,
        gated(system, move |held: &State<S>| held.current == state),
    )
}
