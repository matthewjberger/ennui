use crate::resources::Input;
use ennui::prelude::ResMut;

pub(crate) fn clear_frame_input(mut input: ResMut<Input>) {
    input.pressed.clear();
    input.repeated.clear();
    input.buttons_pressed.clear();
    input.pointer_motion = [0.0, 0.0];
    input.scroll = 0.0;
    input.swiped = 0.0;
    input.typed.clear();
    input.pasted = None;
}
