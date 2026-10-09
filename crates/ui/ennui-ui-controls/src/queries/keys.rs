use ennui_platform::prelude::{Input, KeyCode, commanding};

pub fn chords(input: &Input) -> (bool, bool) {
    (
        commanding(input),
        input.held.contains(&KeyCode::ShiftLeft) || input.held.contains(&KeyCode::ShiftRight),
    )
}

pub fn struck(input: &Input, key: KeyCode) -> bool {
    input.pressed.contains(&key) || input.repeated.contains(&key)
}
