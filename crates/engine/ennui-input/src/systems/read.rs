use crate::commands::gather::gather;
use crate::data::{Action, Heard};
use crate::resources::{Actions, Bindings};
use ennui::prelude::{Res, ResMut};
use ennui_platform::prelude::{Claimed, Input};

pub fn read<A: Action>(
    input: Res<Input>,
    claimed: Res<Claimed>,
    bindings: Res<Bindings<A>>,
    mut actions: ResMut<Actions<A>>,
) {
    let heard = Heard {
        input: &input,
        pointed: !claimed.pointer.is_empty(),
        typing: !claimed.keys.is_empty(),
    };
    gather(&bindings.worn, &mut actions, heard);
}
