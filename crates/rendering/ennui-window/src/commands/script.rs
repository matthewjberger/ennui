use ennui_platform::data::{Act, Deed};
use ennui_platform::prelude::Input;
use winit::event::MouseButton;

pub(crate) fn play_script(input: &mut Input, deeds: &[Deed], frame: u64) {
    let mut seen = deeds.iter().any(|deed| {
        deed.from < frame && matches!(deed.act, Act::Click(_) | Act::Point(_) | Act::Drag { .. })
    });
    for deed in deeds {
        let during = deed.from <= frame && frame <= deed.to;
        let starts = frame == deed.from;
        let ends = frame == deed.to + 1;
        match &deed.act {
            Act::Key(key) => {
                let key = *key;
                if starts && input.held.insert(key) {
                    input.pressed.insert(key);
                }
                if ends
                    && !covered(
                        deeds,
                        frame,
                        |act| matches!(act, Act::Key(held) if *held == key),
                    )
                {
                    input.held.remove(&key);
                }
            }
            Act::Click(at) => {
                if starts {
                    place_pointer(input, *at, &mut seen);
                    if input.buttons_held.insert(MouseButton::Left) {
                        input.buttons_pressed.insert(MouseButton::Left);
                    }
                }
                if ends && !covered(deeds, frame, |act| matches!(act, Act::Click(_))) {
                    input.buttons_held.remove(&MouseButton::Left);
                }
            }
            Act::Point(at) => {
                if starts {
                    place_pointer(input, *at, &mut seen);
                }
            }
            Act::Wheel(steps) => {
                if during {
                    input.scroll += steps;
                }
            }
            Act::Drag { button, from, to } => {
                if starts {
                    place_pointer(input, *from, &mut seen);
                    if input.buttons_held.insert(*button) {
                        input.buttons_pressed.insert(*button);
                    }
                }
                if during {
                    let span = (deed.to - deed.from).max(1) as f32;
                    let share = (frame - deed.from) as f32 / span;
                    let at = [
                        from[0] + (to[0] - from[0]) * share,
                        from[1] + (to[1] - from[1]) * share,
                    ];
                    place_pointer(input, at, &mut seen);
                }
                if ends
                    && !covered(
                        deeds,
                        frame,
                        |act| matches!(act, Act::Drag { button: held, .. } if held == button),
                    )
                {
                    input.buttons_held.remove(button);
                }
            }
            Act::Type(written) => {
                if starts {
                    input
                        .typed
                        .extend(written.chars().filter(|held| !held.is_control()));
                }
            }
        }
    }
}

fn place_pointer(input: &mut Input, at: [f32; 2], seen: &mut bool) {
    if *seen {
        input.pointer_motion[0] += at[0] - input.pointer[0];
        input.pointer_motion[1] += at[1] - input.pointer[1];
    }
    input.pointer = at;
    input.pointer_inside = true;
    *seen = true;
}

fn covered(deeds: &[Deed], frame: u64, same: impl Fn(&Act) -> bool) -> bool {
    deeds
        .iter()
        .any(|deed| deed.from <= frame && frame <= deed.to && same(&deed.act))
}
