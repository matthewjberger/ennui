use crate::data::{Gesture, Host, Touching};
use crate::theme::{COAST_DECAY, COAST_STOP, SPEED_BLEND, TOUCH_SLOP, WHEEL_PIXELS_EACH_LINE};
use ennui::events::send;
use ennui::prelude::Events;
use ennui::resources::get_mut;
use ennui_platform::prelude::{Dropped, FileDropped, Focused, Input, gather_dropped};
use std::path::PathBuf;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, Touch, TouchPhase};
use winit::keyboard::PhysicalKey;

pub(crate) fn take_file(host: &mut Host, path: PathBuf) {
    let mut found = Vec::new();
    gather_dropped(&path, &mut found);
    get_mut::<Dropped>(&mut host.app.resources).hovering = false;
    let dropped_files = get_mut::<Events<FileDropped>>(&mut host.app.resources);
    for held in found {
        send(dropped_files, FileDropped(held));
    }
}

pub(crate) fn move_pointer(host: &mut Host, at: [f32; 2]) {
    let seen = host.pointer_seen;
    host.pointer_seen = true;
    let input = get_mut::<Input>(&mut host.app.resources);
    if seen {
        input.pointer_motion[0] += at[0] - input.pointer[0];
        input.pointer_motion[1] += at[1] - input.pointer[1];
    }
    input.pointer = at;
    input.pointer_inside = true;
}

pub(crate) fn take_focus(host: &mut Host, on: bool) {
    get_mut::<Focused>(&mut host.app.resources).0 = on;
    if on {
        return;
    }
    let input = get_mut::<Input>(&mut host.app.resources);
    input.held.clear();
    input.buttons_held.clear();
}

pub(crate) fn turn_wheel(host: &mut Host, delta: MouseScrollDelta) {
    let steps = match delta {
        MouseScrollDelta::LineDelta(_, lines) => lines,
        MouseScrollDelta::PixelDelta(moved) => moved.y as f32 / WHEEL_PIXELS_EACH_LINE,
    };
    get_mut::<Input>(&mut host.app.resources).scroll += steps;
}

pub(crate) fn press_button(host: &mut Host, state: ElementState, button: MouseButton) {
    let input = get_mut::<Input>(&mut host.app.resources);
    match state {
        ElementState::Pressed => {
            if input.buttons_held.insert(button) {
                input.buttons_pressed.insert(button);
            }
        }
        ElementState::Released => {
            input.buttons_held.remove(&button);
        }
    }
}

pub(crate) fn touch(host: &mut Host, touched: Touch) {
    if host.touching.finger.is_some_and(|held| held != touched.id) {
        return;
    }
    let at = [touched.location.x as f32, touched.location.y as f32];
    match touched.phase {
        TouchPhase::Started => {
            host.touching = Touching {
                finger: Some(touched.id),
                start: at,
                last: at,
                ..Touching::default()
            };
            move_pointer(host, at);
        }
        TouchPhase::Moved => {
            let start = host.touching.start;
            let across = at[0] - start[0];
            let down = at[1] - start[1];
            if host.touching.gesture == Gesture::Waiting && across.hypot(down) > TOUCH_SLOP {
                match down.abs() > across.abs() {
                    true => host.touching.gesture = Gesture::Scrolling,
                    false => {
                        host.touching.gesture = Gesture::Pressing;
                        press_button(host, ElementState::Pressed, MouseButton::Left);
                    }
                }
            }
            match host.touching.gesture {
                Gesture::Scrolling => {
                    let step = at[1] - host.touching.last[1];
                    get_mut::<Input>(&mut host.app.resources).swiped += step;
                    host.touching.moved += step;
                }
                Gesture::Pressing => move_pointer(host, at),
                Gesture::Waiting => {}
            }
            host.touching.last = at;
        }
        TouchPhase::Ended | TouchPhase::Cancelled => {
            match host.touching.gesture {
                Gesture::Waiting if touched.phase == TouchPhase::Ended => {
                    press_button(host, ElementState::Pressed, MouseButton::Left);
                    host.touching.lift = true;
                }
                Gesture::Waiting | Gesture::Scrolling => {}
                Gesture::Pressing => press_button(host, ElementState::Released, MouseButton::Left),
            }
            if host.touching.gesture != Gesture::Scrolling {
                host.touching.speed = 0.0;
            }
            host.touching.finger = None;
            host.touching.gesture = Gesture::Waiting;
            #[cfg(target_arch = "wasm32")]
            crate::commands::web::raise_keyboard(host);
        }
    }
}

pub(crate) fn coast(host: &mut Host, span: f32) {
    let touching = &mut host.touching;
    if touching.finger.is_some() {
        if span > 0.0 {
            let speed = touching.moved / span;
            touching.speed += (speed - touching.speed) * SPEED_BLEND;
        }
        touching.moved = 0.0;
        return;
    }
    if touching.speed.abs() < COAST_STOP {
        touching.speed = 0.0;
        return;
    }
    let step = touching.speed * span;
    touching.speed *= (-COAST_DECAY * span).exp();
    get_mut::<Input>(&mut host.app.resources).swiped += step;
}

pub(crate) fn lift(host: &mut Host) {
    if std::mem::take(&mut host.touching.lift) {
        press_button(host, ElementState::Released, MouseButton::Left);
    }
}
pub(crate) fn press_key(host: &mut Host, event: KeyEvent) {
    if event.state == ElementState::Pressed
        && let Some(written) = event.text.as_ref()
    {
        let input = get_mut::<Input>(&mut host.app.resources);
        input
            .typed
            .extend(written.chars().filter(|held| !held.is_control()));
    }
    let PhysicalKey::Code(code) = event.physical_key else {
        return;
    };
    let input = get_mut::<Input>(&mut host.app.resources);
    match event.state {
        ElementState::Pressed => {
            if input.held.insert(code) {
                input.pressed.insert(code);
                #[cfg(not(target_arch = "wasm32"))]
                if code == winit::keyboard::KeyCode::KeyV
                    && ennui_platform::prelude::commanding(input)
                {
                    input.pasted = ennui_platform::prelude::read_clipboard();
                }
            } else {
                input.repeated.insert(code);
            }
        }
        ElementState::Released => {
            input.held.remove(&code);
        }
    }
}
