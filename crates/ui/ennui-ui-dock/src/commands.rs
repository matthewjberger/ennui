use crate::components::{Board, Dragged, Leafed, Sash, Zone};
use crate::data::{Shown, Tile, Tiles, Way};
use crate::queries::{
    content_of, corner, drop_rect, halves, parent_of, sash, strip_of, turn_to, under,
};
use crate::theme::{GRAB, MARK, TAB_SIDE};
use ennui::later::{attach, change, set_if_new};
use ennui::prelude::{Edits, Entity, Later, Mut, Storage, peek_mut};
use ennui::storage::{Stamp, get};
use ennui_platform::prelude::CursorIcon;
use ennui_ui::prelude::{
    Centered, Cursor, Dye, Float, Frame, Hidden, Lay, Line, Lit, Order, Panel, Rect, Span, Theme,
    Tone, afloat, frame, heighten, ink, keyed, label, panel, relay, renew, shielded, spacing,
    sweep, touched, widen,
};

use ennui_ui_controls::prelude::hint;
use nalgebra_glm::{Vec2, Vec4};

pub fn board(later: &mut Later, look: &Theme, parent: Entity) -> Entity {
    let held = panel(
        later,
        parent,
        Frame::new(look)
            .wide(Span::Fill(1.0))
            .tall(Span::Fill(1.0))
            .bare()
            .round(0.0)
            .pad(0.0)
            .gap(0.0),
    );
    let mark = frame(
        later,
        Frame::new(look)
            .wide(Span::Fixed(0.0))
            .tall(Span::Fixed(0.0))
            .fill(Vec4::new(0.0, 0.0, 0.0, 0.0))
            .round(look.round * 0.5)
            .pad(0.0),
    );
    afloat(later, held, mark);
    attach(
        later,
        mark,
        (Hidden(true), Order(u32::MAX), Tone::filled(look.chosen)),
    );
    attach(
        later,
        held,
        (Board(Tiles::new()), Dragged::default(), Zone(mark)),
    );
    held
}

pub fn pane(edits: &mut Edits, held: Entity, content: Entity, (name, tip): (&str, &str)) {
    let (name, tip) = (String::from(name), String::from(tip));
    change(edits, move |storage| {
        let Some(board) = get::<Board>(&*storage, held).cloned() else {
            return;
        };
        let mut tiles = board.0;
        let pane = add_tile(
            &mut tiles,
            Tile::Pane {
                held: content,
                name,
                tip,
                slim: false,
            },
        );
        let root = tiles.root;
        if let Some(Some(Tile::Tabs { panes, .. })) = tiles.held.get_mut(root) {
            panes.push(pane);
        }
        ennui::storage::set(&mut *storage, held, Board(tiles));
    });
    afloat(edits, held, content);
}

pub(crate) fn make_sashes(
    seen: &Storage,
    later: &mut Later,
    look: &Theme,
    board: Entity,
    (tiles, gap, least): (&Tiles, f32, Vec2),
) {
    if tiles.full.is_some() {
        return;
    }
    let splits: Vec<(usize, bool, f32)> = tiles
        .held
        .iter()
        .enumerate()
        .filter_map(|(at, held)| match held {
            Some(Tile::Split { down, share, .. }) => Some((at, *down, *share)),
            _ => None,
        })
        .collect();
    for (tile, down, share) in splits {
        let rect = sash(
            tiles.laid.get(tile).copied().unwrap_or_default(),
            down,
            share,
            gap,
            least,
        );
        if rect.size.x <= 0.0 || rect.size.y <= 0.0 {
            continue;
        }
        let bar = keyed(seen, later, board, tile as u64, |later| {
            let held = panel(
                later,
                board,
                Frame::new(look).role(Dye::Edge).round(0.0).pad(0.0),
            );
            afloat(later, board, held);
            touched(later, held);
            held
        });
        set_if_new(later, bar, Sash { board, tile });
        resize(seen, later, bar, rect.size);
        set_if_new(later, bar, Float(corner(rect)));
        set_if_new(
            later,
            bar,
            Cursor(match down {
                true => CursorIcon::NsResize,
                false => CursorIcon::EwResize,
            }),
        );
    }
}

pub(crate) fn make_strips(
    seen: &Storage,
    later: &mut Later,
    look: &Theme,
    board: Entity,
    tiles: &Tiles,
) {
    let groups: Vec<(usize, Vec<usize>, usize)> = match tiles.full {
        Some(full) => vec![(tiles.root, vec![full], 0)],
        None => tiles
            .held
            .iter()
            .enumerate()
            .filter_map(|(at, held)| match held {
                Some(Tile::Tabs { panes, at: on }) => Some((at, panes.clone(), *on)),
                _ => None,
            })
            .collect(),
    };
    for (tile, panes, on) in groups {
        let rect = tiles.laid.get(tile).copied().unwrap_or_default();
        if rect.size.x <= 0.0 {
            continue;
        }
        let strip = match tiles.full {
            Some(full) => strip_of(tiles, full),
            None => strip_of(tiles, tile),
        };
        let bar = keyed(seen, later, board, 0x1000_0000 + tile as u64, |later| {
            let held = panel(
                later,
                board,
                Frame::new(look)
                    .flow(Lay::Row)
                    .along(Line::Start)
                    .role(Dye::Input)
                    .round(0.0)
                    .pad(0.0)
                    .gap(0.0),
            );
            afloat(later, board, held);
            relay(later, held, |panel| panel.clips = true);
            held
        });
        resize(seen, later, bar, Vec2::new(rect.size.x, strip));
        set_if_new(later, bar, Float(corner(rect)));
        let caption = look.caption;
        let mark = MARK;
        for (slot, pane) in panes.iter().enumerate() {
            let (name, tip) = match tiles.held.get(*pane).and_then(Option::as_ref) {
                Some(Tile::Pane { name, tip, .. }) => (name.clone(), tip.clone()),
                _ => (String::new(), String::new()),
            };
            if slot > 0 {
                let line = keyed(seen, later, bar, 0x2000_0000 + *pane as u64, |later| {
                    panel(
                        later,
                        bar,
                        Frame::new(look)
                            .wide(Span::Fixed(look.line))
                            .tall(Span::Fill(1.0))
                            .role(Dye::Edge)
                            .round(0.0)
                            .pad(0.0)
                            .gap(0.0),
                    )
                });
                set_if_new(later, line, Order(slot as u32 * 2 - 1));
            }
            let tab = keyed(seen, later, bar, *pane as u64, |later| {
                let held = panel(
                    later,
                    bar,
                    Frame::new(look)
                        .tall(Span::Fill(1.0))
                        .along(Line::Start)
                        .roles(Dye::Input, Dye::Panel)
                        .round(0.0)
                        .pad(0.0)
                        .gap(0.0),
                );
                relay(later, held, |panel| {
                    panel.clips = true;
                    panel.shrinks = true;
                });
                touched(later, held);
                shielded(later, held);
                hint(later, held, &tip);
                held
            });
            let lit = keyed(seen, later, tab, 1, |later| {
                panel(
                    later,
                    tab,
                    Frame::new(look)
                        .wide(Span::Fill(1.0))
                        .tall(Span::Fixed(mark))
                        .role(Dye::Accent)
                        .round(0.0)
                        .pad(0.0)
                        .gap(0.0),
                )
            });
            let words = keyed(seen, later, tab, 2, |later| {
                let held = panel(
                    later,
                    tab,
                    Frame::new(look)
                        .flow(Lay::Row)
                        .along(Line::Middle)
                        .across(Line::Middle)
                        .tall(Span::Fill(1.0))
                        .bare()
                        .pad(0.0)
                        .gap(0.0),
                );
                relay(later, held, |panel| {
                    panel.clips = true;
                    panel.shrinks = true;
                });
                spacing(later, look, held, look.pad * TAB_SIDE);
                held
            });
            let text = keyed(seen, later, words, 3, |later| {
                let text = label(later, look, words, &name, caption);
                set_if_new(later, text, Centered);
                ink(later, text, Dye::Faint, Dye::Ink);
                text
            });
            keyed(seen, later, words, 4, |later| {
                spacing(later, look, words, look.pad * TAB_SIDE)
            });
            set_if_new(later, tab, Leafed { board, tile: *pane });
            set_if_new(later, tab, Lit(slot == on));
            set_if_new(later, lit, Hidden(slot != on));
            set_if_new(later, text, Lit(slot == on));
            set_if_new(later, tab, Order(slot as u32 * 2));
        }
        sweep(later, bar);
    }
}

pub(crate) fn drag_board(
    tiles: &mut Stamp<'_, Board>,
    dragged: &mut Stamp<'_, Dragged>,
    grabbed: Option<usize>,
    at: Vec2,
    down: bool,
) -> Option<Shown> {
    if let Some(pane) = grabbed {
        if let Some(turned) = turn_to(&tiles.0, pane) {
            **tiles = Board(turned);
        }
        **dragged = Dragged {
            pane: Some(pane),
            from: at,
            live: false,
        };
        return None;
    }
    let held = **dragged;
    let pane = held.pane?;
    let live = held.live || (at - held.from).magnitude() > GRAB;
    let target = match live {
        true => under(&tiles.0, at),
        false => None,
    };
    if !down {
        **dragged = Dragged::default();
        if let Some((tile, way)) = target {
            let mut wanted = tiles.0.clone();
            drop_into(&mut wanted, pane, tile, way);
            **tiles = Board(wanted);
        }
        return Some(Shown::Dropped);
    }
    renew(
        dragged,
        Dragged {
            pane: Some(pane),
            from: held.from,
            live,
        },
    );
    Some(match target {
        Some((tile, way)) => Shown::Over(drop_rect(
            content_of(
                tiles.0.laid.get(tile).copied().unwrap_or_default(),
                strip_of(&tiles.0, tile),
            ),
            way,
        )),
        None => Shown::Missed,
    })
}

pub(crate) fn show_zones(
    marks: &mut Mut<(Hidden, Panel, Float, Tone)>,
    shown: Vec<(Entity, Shown)>,
    shade: Vec4,
) {
    for (entity, wanted) in shown {
        let Some((mut hidden, mut panel, mut float, mut tone)) = peek_mut(marks, entity) else {
            continue;
        };
        match wanted {
            Shown::Dropped => *hidden = Hidden(true),
            Shown::Missed => renew(&mut hidden, Hidden(true)),
            Shown::Over(rect) => {
                renew(&mut hidden, Hidden(false));
                widen(&mut panel, Span::Fixed(rect.size.x));
                heighten(&mut panel, Span::Fixed(rect.size.y));
                renew(&mut float, Float(corner(rect)));
                renew(&mut tone, Tone::filled(shade));
            }
        }
    }
}

pub fn add_tile(tiles: &mut Tiles, tile: Tile) -> usize {
    tiles.held.push(Some(tile));
    tiles.laid.push(Rect::default());
    tiles.held.len() - 1
}

pub(crate) fn resize(seen: &Storage, later: &mut Later, entity: Entity, size: Vec2) {
    let wanted = (Span::Fixed(size.x), Span::Fixed(size.y));
    if get::<Panel>(seen, entity).is_some_and(|panel| (panel.wide, panel.tall) == wanted) {
        return;
    }
    relay(later, entity, move |panel| {
        panel.wide = wanted.0;
        panel.tall = wanted.1;
    });
}

pub(crate) fn lay(tiles: &Tiles, laid: &mut [Rect], rect: Rect, (gap, least): (f32, Vec2)) {
    laid.fill(Rect::default());
    match tiles.full {
        Some(full) => {
            let strip = strip_of(tiles, full);
            if let Some(held) = laid.get_mut(tiles.root) {
                *held = rect;
            }
            if let Some(held) = laid.get_mut(full) {
                *held = content_of(rect, strip);
            }
        }
        None => lay_tile(tiles, laid, tiles.root, rect, (gap, least)),
    }
}

fn lay_tile(tiles: &Tiles, laid: &mut [Rect], id: usize, rect: Rect, (gap, least): (f32, Vec2)) {
    if let Some(held) = laid.get_mut(id) {
        *held = rect;
    }
    match tiles.held.get(id).and_then(Option::as_ref) {
        Some(Tile::Split { down, share, kids }) => {
            let (first, second) = halves(rect, *down, *share, gap, least);
            lay_tile(tiles, laid, kids[0], first, (gap, least));
            lay_tile(tiles, laid, kids[1], second, (gap, least));
        }
        Some(Tile::Tabs { panes, at }) => {
            let room = content_of(rect, strip_of(tiles, id));
            if let Some(pane) = panes.get(*at) {
                lay_tile(tiles, laid, *pane, room, (gap, least));
            }
        }
        _ => {}
    }
}

pub(crate) fn prune(tiles: &mut Tiles) {
    loop {
        let root = tiles.root;
        let empties: Vec<usize> = tiles
            .held
            .iter()
            .enumerate()
            .filter_map(|(id, held)| match held {
                Some(Tile::Tabs { panes, .. }) if panes.is_empty() && id != root => Some(id),
                _ => None,
            })
            .collect();
        let mut changed = false;
        for dead in empties {
            let Some(above) = parent_of(tiles, dead) else {
                continue;
            };
            let kept = match tiles.held.get(above).and_then(Option::as_ref) {
                Some(Tile::Split { kids, .. }) => match kids[0] == dead {
                    true => kids[1],
                    false => kids[0],
                },
                _ => continue,
            };
            let Some(held) = tiles.held.get(kept).and_then(|held| held.clone()) else {
                continue;
            };
            tiles.held[above] = Some(held);
            tiles.held[kept] = None;
            tiles.held[dead] = None;
            changed = true;
        }
        if !changed {
            return;
        }
    }
}

fn take_out(tiles: &mut Tiles, pane: usize) -> Option<usize> {
    let owner = parent_of(tiles, pane);
    if let Some(Some(Tile::Tabs { panes, at })) = owner.and_then(|owner| tiles.held.get_mut(owner))
    {
        panes.retain(|held| *held != pane);
        *at = (*at).min(panes.len().saturating_sub(1));
    }
    owner
}

pub fn drop_into(tiles: &mut Tiles, pane: usize, target: usize, way: Way) {
    let target = match tiles.held.get(target).and_then(Option::as_ref) {
        Some(Tile::Pane { .. }) => match parent_of(tiles, target) {
            Some(held) => held,
            None => return,
        },
        Some(_) => target,
        None => return,
    };
    if target == pane {
        return;
    }
    let owner = parent_of(tiles, pane);
    let lone = matches!(tiles.held.get(target).and_then(Option::as_ref), Some(Tile::Tabs { panes, .. }) if panes.len() <= 1);
    if owner == Some(target) && (way == Way::Onto || lone) {
        return;
    }
    take_out(tiles, pane);
    match way {
        Way::Onto => {
            if let Some(Some(Tile::Tabs { panes, at })) = tiles.held.get_mut(target) {
                panes.push(pane);
                *at = panes.len() - 1;
            }
        }
        _ => {
            let Some(kept) = tiles.held.get(target).and_then(|held| held.clone()) else {
                return;
            };
            let moved = add_tile(tiles, kept);
            let fresh = add_tile(
                tiles,
                Tile::Tabs {
                    panes: vec![pane],
                    at: 0,
                },
            );
            let down = matches!(way, Way::Above | Way::Below);
            let first = matches!(way, Way::Left | Way::Above);
            tiles.held[target] = Some(Tile::Split {
                down,
                share: 0.5,
                kids: match first {
                    true => [fresh, moved],
                    false => [moved, fresh],
                },
            });
        }
    }
    prune(tiles);
}

pub fn close_pane(tiles: &mut Tiles, pane: usize) {
    if tiles.full == Some(pane) {
        tiles.full = None;
    }
    take_out(tiles, pane);
    prune(tiles);
}

pub fn open_pane(tiles: &mut Tiles, pane: usize, mates: &[usize]) {
    tiles.full = None;
    if parent_of(tiles, pane).is_some() {
        if let Some(turned) = turn_to(tiles, pane) {
            *tiles = turned;
        }
        return;
    }
    let group = mates.iter().find_map(|mate| parent_of(tiles, *mate));
    match group {
        Some(group) => {
            if let Some(Some(Tile::Tabs { panes, at })) = tiles.held.get_mut(group) {
                panes.push(pane);
                *at = panes.len() - 1;
            }
        }
        None => {
            let root = tiles.root;
            drop_into(tiles, pane, root, Way::Right);
        }
    }
}

pub fn maximize(tiles: &mut Tiles, pane: usize) {
    tiles.full = match tiles.full {
        Some(_) => None,
        None => Some(pane),
    };
}
