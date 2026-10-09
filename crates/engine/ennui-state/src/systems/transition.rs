use crate::data::States;
use crate::resources::State;
use ennui::prelude::ResMut;

pub fn apply<S: States>(mut state: ResMut<State<S>>) {
    let Some(next) = state.next.take() else {
        return;
    };
    state.from = Some(state.current);
    state.turn += 1;
    state.current = next;
}
