use crate::data::{COMMANDS, Chord, Command};
use ennui_platform::prelude::{Input, KeyCode};

pub(crate) fn modifiers(input: &Input) -> (bool, bool, bool) {
    let held = |keys: [KeyCode; 2]| keys.iter().any(|key| input.held.contains(key));
    (
        held([KeyCode::ShiftLeft, KeyCode::ShiftRight]),
        held([KeyCode::ControlLeft, KeyCode::ControlRight]),
        held([KeyCode::AltLeft, KeyCode::AltRight]),
    )
}

fn chord_now(input: &Input) -> Option<Chord> {
    let (shift, control, alt) = modifiers(input);
    match (control, shift, alt) {
        (false, false, false) => Some(Chord::Bare),
        (true, false, false) => Some(Chord::Control),
        (false, true, false) => Some(Chord::Shift),
        (false, false, true) => Some(Chord::Alt),
        (true, true, false) => Some(Chord::ControlShift),
        _ => None,
    }
}

pub(crate) fn pressed_actions(input: &Input) -> Vec<Command> {
    let Some(chord) = chord_now(input) else {
        return Vec::new();
    };
    COMMANDS
        .iter()
        .filter(|(_, _, bindings)| {
            bindings
                .iter()
                .any(|(wanted, key)| *wanted == chord && input.pressed.contains(key))
        })
        .map(|(action, _, _)| *action)
        .collect()
}

pub(crate) fn menu_line(wanted: Command) -> String {
    let label = COMMANDS
        .iter()
        .find(|(action, _, _)| *action == wanted)
        .map_or("", |(_, label, _)| label);
    match keys_of(wanted) {
        keys if keys.is_empty() => String::from(label),
        keys => format!("{label}\t{keys}"),
    }
}

pub(crate) fn with_keys(said: &str, wanted: Command) -> String {
    match keys_of(wanted) {
        keys if keys.is_empty() => String::from(said),
        keys => format!("{said} ({keys})"),
    }
}

fn key_name(key: KeyCode) -> String {
    let written = format!("{key:?}");
    match key {
        KeyCode::BracketLeft => String::from("["),
        KeyCode::BracketRight => String::from("]"),
        _ => match written.strip_prefix("Numpad") {
            Some(rest) => format!("Numpad {rest}"),
            None => written
                .strip_prefix("Key")
                .or_else(|| written.strip_prefix("Digit"))
                .unwrap_or(&written)
                .to_string(),
        },
    }
}

pub(crate) fn keys_of(wanted: Command) -> String {
    let Some((_, _, bindings)) = COMMANDS.iter().find(|(action, _, _)| *action == wanted) else {
        return String::new();
    };
    bindings
        .iter()
        .map(|(chord, key)| {
            let held = match chord {
                Chord::Bare => "",
                Chord::Control => "Ctrl ",
                Chord::Shift => "Shift ",
                Chord::Alt => "Alt ",
                Chord::ControlShift => "Ctrl Shift ",
            };
            format!("{held}{}", key_name(*key))
        })
        .collect::<Vec<String>>()
        .join(", ")
}
