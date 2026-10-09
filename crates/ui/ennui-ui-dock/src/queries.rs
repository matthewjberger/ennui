use crate::components::Leafed;
use crate::data::{Tile, Tiles, Way};
use crate::theme::{EDGE, SLIM, STRIP};
use ennui::prelude::{Entity, Storage};
use ennui::storage::query;
use ennui_ui::prelude::{Hover, Rect};

use ennui_ui::queries::press::inside_of;
use nalgebra_glm::Vec2;
use std::collections::{HashMap, HashSet};

pub(crate) fn corner(rect: Rect) -> Vec2 {
    Vec2::new(
        rect.center.x - rect.size.x * 0.5,
        rect.center.y + rect.size.y * 0.5,
    )
}

pub(crate) fn turn_to(held: &Tiles, pane: usize) -> Option<Tiles> {
    let mut tiles = held.clone();
    let owner = parent_of(&tiles, pane)?;
    if let Some(Some(Tile::Tabs { panes, at })) = tiles.held.get_mut(owner)
        && let Some(slot) = panes.iter().position(|held| *held == pane)
    {
        *at = slot;
    }
    Some(tiles)
}

fn group_under(tiles: &Tiles, at: Vec2) -> Option<usize> {
    tiles.held.iter().enumerate().find_map(|(id, held)| {
        let rect = tiles.laid.get(id).copied().unwrap_or_default();
        (matches!(held, Some(Tile::Tabs { .. })) && inside_of(at, &rect) && rect.size.x > 0.0)
            .then_some(id)
    })
}

pub(crate) fn strip_of(tiles: &Tiles, group: usize) -> f32 {
    let slim = match tiles.held.get(group).and_then(Option::as_ref) {
        Some(Tile::Tabs { panes, .. }) => {
            !panes.is_empty()
                && panes.iter().all(|pane| {
                    matches!(
                        tiles.held.get(*pane).and_then(Option::as_ref),
                        Some(Tile::Pane { slim: true, .. })
                    )
                })
        }
        Some(Tile::Pane { slim, .. }) => *slim,
        _ => false,
    };
    match slim {
        true => SLIM,
        false => STRIP,
    }
}

pub(crate) fn under(tiles: &Tiles, at: Vec2) -> Option<(usize, Way)> {
    if tiles.full.is_some() {
        return None;
    }
    let held = group_under(tiles, at)?;
    let strip = strip_of(tiles, held);
    let rect = tiles.laid.get(held).copied().unwrap_or_default();
    let top = rect.center.y + rect.size.y * 0.5;
    match at.y > top - strip {
        true => Some((held, Way::Onto)),
        false => Some((held, way_of(content_of(rect, strip), at))),
    }
}

pub fn pane_under(tiles: &Tiles, at: Vec2) -> Option<usize> {
    if let Some(full) = tiles.full {
        let root = tiles.laid.get(tiles.root).copied().unwrap_or_default();
        return inside_of(at, &root).then_some(full);
    }
    match tiles
        .held
        .get(group_under(tiles, at)?)
        .and_then(Option::as_ref)
    {
        Some(Tile::Tabs { panes, at }) => panes.get(*at).copied(),
        _ => None,
    }
}

pub fn tab_under(seen: &Storage, board: Entity) -> Option<usize> {
    query::<(&Leafed, &Hover)>(seen)
        .find(|(_, (leafed, hover))| leafed.board == board && hover.0)
        .map(|(_, (leafed, _))| leafed.tile)
}

pub fn parent_of(tiles: &Tiles, id: usize) -> Option<usize> {
    tiles
        .held
        .iter()
        .enumerate()
        .find_map(|(at, held)| match held {
            Some(Tile::Split { kids, .. }) if kids.contains(&id) => Some(at),
            Some(Tile::Tabs { panes, .. }) if panes.contains(&id) => Some(at),
            _ => None,
        })
}

pub fn panes_of(tiles: &Tiles) -> Vec<(usize, Entity)> {
    tiles
        .held
        .iter()
        .enumerate()
        .filter_map(|(at, held)| match held {
            Some(Tile::Pane { held, .. }) => Some((at, *held)),
            _ => None,
        })
        .collect()
}

pub fn pane_shown(tiles: &Tiles, pane: usize) -> bool {
    if let Some(full) = tiles.full {
        return full == pane;
    }
    match parent_of(tiles, pane).and_then(|at| tiles.held.get(at).and_then(Option::as_ref)) {
        Some(Tile::Tabs { panes, at }) => panes.get(*at) == Some(&pane),
        _ => false,
    }
}

pub(crate) fn share_range(rect: Rect, down: bool, gap: f32, least: Vec2) -> (f32, f32) {
    let (along, low) = match down {
        true => (rect.size.y - gap, least.y),
        false => (rect.size.x - gap, least.x),
    };
    if along <= 0.0 {
        return (0.5, 0.5);
    }
    let low = (low / along).min(0.5);
    (low, 1.0 - low)
}

pub(crate) fn halves(rect: Rect, down: bool, share: f32, gap: f32, least: Vec2) -> (Rect, Rect) {
    let (low, high) = share_range(rect, down, gap, least);
    let share = share.clamp(low, high);
    match down {
        true => {
            let first = (rect.size.y - gap) * share;
            let second = (rect.size.y - gap) - first;
            (
                Rect {
                    center: Vec2::new(
                        rect.center.x,
                        rect.center.y + rect.size.y * 0.5 - first * 0.5,
                    ),
                    size: Vec2::new(rect.size.x, first),
                },
                Rect {
                    center: Vec2::new(
                        rect.center.x,
                        rect.center.y - rect.size.y * 0.5 + second * 0.5,
                    ),
                    size: Vec2::new(rect.size.x, second),
                },
            )
        }
        false => {
            let first = (rect.size.x - gap) * share;
            let second = (rect.size.x - gap) - first;
            (
                Rect {
                    center: Vec2::new(
                        rect.center.x - rect.size.x * 0.5 + first * 0.5,
                        rect.center.y,
                    ),
                    size: Vec2::new(first, rect.size.y),
                },
                Rect {
                    center: Vec2::new(
                        rect.center.x + rect.size.x * 0.5 - second * 0.5,
                        rect.center.y,
                    ),
                    size: Vec2::new(second, rect.size.y),
                },
            )
        }
    }
}

pub(crate) fn sash(rect: Rect, down: bool, share: f32, gap: f32, least: Vec2) -> Rect {
    let (first, _) = halves(rect, down, share, gap, least);
    match down {
        true => Rect {
            center: Vec2::new(
                rect.center.x,
                first.center.y - first.size.y * 0.5 - gap * 0.5,
            ),
            size: Vec2::new(rect.size.x, gap),
        },
        false => Rect {
            center: Vec2::new(
                first.center.x + first.size.x * 0.5 + gap * 0.5,
                rect.center.y,
            ),
            size: Vec2::new(gap, rect.size.y),
        },
    }
}

pub(crate) fn way_of(rect: Rect, at: Vec2) -> Way {
    let away = at - rect.center;
    let across = away.x / rect.size.x.max(f32::EPSILON);
    let down = away.y / rect.size.y.max(f32::EPSILON);
    if across.abs() < EDGE && down.abs() < EDGE {
        return Way::Onto;
    }
    match across.abs() > down.abs() {
        true => match across < 0.0 {
            true => Way::Left,
            false => Way::Right,
        },
        false => match down > 0.0 {
            true => Way::Above,
            false => Way::Below,
        },
    }
}

pub(crate) fn drop_rect(rect: Rect, way: Way) -> Rect {
    let half = Vec2::new(rect.size.x * 0.5, rect.size.y * 0.5);
    match way {
        Way::Onto => rect,
        Way::Left => Rect {
            center: Vec2::new(rect.center.x - half.x * 0.5, rect.center.y),
            size: Vec2::new(half.x, rect.size.y),
        },
        Way::Right => Rect {
            center: Vec2::new(rect.center.x + half.x * 0.5, rect.center.y),
            size: Vec2::new(half.x, rect.size.y),
        },
        Way::Above => Rect {
            center: Vec2::new(rect.center.x, rect.center.y + half.y * 0.5),
            size: Vec2::new(rect.size.x, half.y),
        },
        Way::Below => Rect {
            center: Vec2::new(rect.center.x, rect.center.y - half.y * 0.5),
            size: Vec2::new(rect.size.x, half.y),
        },
    }
}

pub(crate) fn content_of(rect: Rect, strip: f32) -> Rect {
    Rect {
        center: Vec2::new(rect.center.x, rect.center.y - strip * 0.5),
        size: Vec2::new(rect.size.x, (rect.size.y - strip).max(0.0)),
    }
}

fn write_tile(tiles: &Tiles, id: usize, out: &mut String) {
    match tiles.held.get(id).and_then(Option::as_ref) {
        Some(Tile::Pane { name, .. }) => out.push_str(name),
        Some(Tile::Split { down, share, kids }) => {
            let way = match down {
                true => "down",
                false => "across",
            };
            out.push_str(&format!("({way} {share:.3} "));
            write_tile(tiles, kids[0], out);
            out.push(' ');
            write_tile(tiles, kids[1], out);
            out.push(')');
        }
        Some(Tile::Tabs { panes, at }) => {
            out.push_str(&format!("(tabs {at}"));
            for pane in panes {
                out.push(' ');
                write_tile(tiles, *pane, out);
            }
            out.push(')');
        }
        None => {}
    }
}

pub fn written(tiles: &Tiles) -> String {
    let mut out = String::new();
    write_tile(tiles, tiles.root, &mut out);
    out
}

struct Reading<'held> {
    tokens: Vec<&'held str>,
    at: usize,
    known: HashMap<&'held str, (Entity, &'held str, bool)>,
    used: HashSet<&'held str>,
    tiles: Tiles,
}

fn read_pane(reading: &mut Reading<'_>, name: &str) -> Option<usize> {
    let (key, (held, tip, slim)) = reading.known.get_key_value(name)?;
    let (key, held, tip, slim) = (*key, *held, *tip, *slim);
    if !reading.used.insert(key) {
        return None;
    }
    let tile = Tile::Pane {
        held,
        name: String::from(name),
        tip: String::from(tip),
        slim,
    };
    reading.tiles.held.push(Some(tile));
    reading.tiles.laid.push(Rect::default());
    Some(reading.tiles.held.len() - 1)
}

fn read_tile(reading: &mut Reading<'_>) -> Option<usize> {
    let token = *reading.tokens.get(reading.at)?;
    reading.at += 1;
    if token != "(" {
        let pane = read_pane(reading, token)?;
        reading.tiles.held.push(Some(Tile::Tabs {
            panes: vec![pane],
            at: 0,
        }));
        reading.tiles.laid.push(Rect::default());
        return Some(reading.tiles.held.len() - 1);
    }
    let kind = *reading.tokens.get(reading.at)?;
    reading.at += 1;
    let made = match kind {
        "tabs" => {
            let at: usize = reading.tokens.get(reading.at)?.parse().ok()?;
            reading.at += 1;
            let mut panes = Vec::new();
            while *reading.tokens.get(reading.at)? != ")" {
                let name = reading.tokens[reading.at];
                reading.at += 1;
                panes.push(read_pane(reading, name)?);
            }
            if panes.is_empty() {
                return None;
            }
            Tile::Tabs {
                at: at.min(panes.len() - 1),
                panes,
            }
        }
        "down" | "across" => {
            let share: f32 = reading.tokens.get(reading.at)?.parse().ok()?;
            reading.at += 1;
            let first = read_tile(reading)?;
            let second = read_tile(reading)?;
            Tile::Split {
                down: kind == "down",
                share: share.clamp(0.0, 1.0),
                kids: [first, second],
            }
        }
        _ => return None,
    };
    if *reading.tokens.get(reading.at)? != ")" {
        return None;
    }
    reading.at += 1;
    reading.tiles.held.push(Some(made));
    reading.tiles.laid.push(Rect::default());
    Some(reading.tiles.held.len() - 1)
}

pub fn read_layout(text: &str, current: &Tiles) -> Option<Tiles> {
    let spaced = text.replace('(', " ( ").replace(')', " ) ");
    let known: HashMap<&str, (Entity, &str, bool)> = current
        .held
        .iter()
        .filter_map(|held| match held {
            Some(Tile::Pane {
                held,
                name,
                tip,
                slim,
            }) => Some((name.as_str(), (*held, tip.as_str(), *slim))),
            _ => None,
        })
        .collect();
    let mut reading = Reading {
        tokens: spaced.split_whitespace().collect(),
        at: 0,
        known,
        used: HashSet::new(),
        tiles: Tiles::default(),
    };
    let root = read_tile(&mut reading)?;
    if reading.at != reading.tokens.len() {
        return None;
    }
    reading.tiles.root = root;
    for held in current.held.iter().flatten() {
        if let Tile::Pane { name, .. } = held
            && !reading.used.contains(name.as_str())
        {
            reading.tiles.held.push(Some(held.clone()));
            reading.tiles.laid.push(Rect::default());
        }
    }
    Some(reading.tiles)
}
