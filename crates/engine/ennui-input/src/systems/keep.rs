use crate::commands::bind::{settled, write};
use crate::data::Action;
use crate::resources::Bindings;
use ennui::prelude::{Res, ResMut};
use ennui_platform::prelude::{Closing, Time};

pub(crate) fn keep<A: Action>(
    time: Res<Time>,
    closing: Res<Closing>,
    mut bindings: ResMut<Bindings<A>>,
) {
    if settled(&mut bindings, time.since_last_frame, closing.0) {
        write(&mut bindings);
    }
}
