use crate::data::{Act, Deed};
use serde::Deserialize;
use serde::de::IntoDeserializer;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

pub(crate) fn pressing(text: &str) -> Result<Deed, String> {
    let (name, frames) = parted(text)?;
    let key =
        KeyCode::deserialize(name.into_deserializer()).map_err(|_: serde::de::value::Error| {
            format!("{name} is not a key name such as KeyJ or Space")
        })?;
    deed(Act::Key(key), frames)
}

pub(crate) fn clicking(text: &str) -> Result<Deed, String> {
    let (at, frames) = parted(text)?;
    deed(Act::Click(spot(at)?), frames)
}

pub(crate) fn typing(text: &str) -> Result<Deed, String> {
    let (written, frames) = text
        .rsplit_once('@')
        .ok_or_else(|| format!("{text} needs a frame after @, such as hello@60"))?;
    deed(Act::Type(String::from(written)), frames)
}

pub(crate) fn pointing(text: &str) -> Result<Deed, String> {
    let (at, frames) = parted(text)?;
    deed(Act::Point(spot(at)?), frames)
}

pub(crate) fn wheeling(text: &str) -> Result<Deed, String> {
    let (steps, frames) = parted(text)?;
    let steps = steps
        .parse::<f32>()
        .map_err(|_| format!("{steps} is not a number of wheel steps"))?;
    deed(Act::Wheel(steps), frames)
}

pub(crate) fn dragging(text: &str) -> Result<Deed, String> {
    let (line, frames) = parted(text)?;
    let (name, places) = line
        .split_once(':')
        .ok_or_else(|| format!("{line} needs a button first, such as Middle:640,360>900,360"))?;
    let button = match name {
        "Left" => MouseButton::Left,
        "Right" => MouseButton::Right,
        "Middle" => MouseButton::Middle,
        _ => return Err(format!("{name} is not Left, Right or Middle")),
    };
    let (from, to) = places
        .split_once('>')
        .ok_or_else(|| format!("{places} needs two places, such as 640,360>900,360"))?;
    deed(
        Act::Drag {
            button,
            from: spot(from)?,
            to: spot(to)?,
        },
        frames,
    )
}

fn parted(text: &str) -> Result<(&str, &str), String> {
    text.split_once('@')
        .ok_or_else(|| format!("{text} needs a frame after @, such as KeyJ@60 or 640,360@60-90"))
}

fn deed(act: Act, frames: &str) -> Result<Deed, String> {
    let (from, to) = frames.split_once('-').unwrap_or((frames, frames));
    let number = |held: &str| {
        held.parse::<u64>()
            .map_err(|_| format!("{held} is not a frame number"))
    };
    let (from, to) = (number(from)?, number(to)?);
    if to < from {
        return Err(format!("the frames {frames} end before they start"));
    }
    Ok(Deed { act, from, to })
}

fn spot(text: &str) -> Result<[f32; 2], String> {
    let (across, down) = text
        .split_once(',')
        .ok_or_else(|| format!("a pointer place reads as X,Y, not {text}"))?;
    let number = |held: &str| {
        held.parse::<f32>()
            .map_err(|_| format!("{held} is not a pixel position"))
    };
    Ok([number(across)?, number(down)?])
}
