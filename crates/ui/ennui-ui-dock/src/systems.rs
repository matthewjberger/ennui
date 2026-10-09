use crate::commands::{drag_board, lay, make_sashes, make_strips, resize, show_zones};
use crate::components::{Board, Leafed, Sash};
use crate::data::{Dragging, Shown, Tile, Tiles};
use crate::queries::{corner, pane_shown, panes_of, share_range};
use crate::theme::{BAR, LEAST_TALL, LEAST_WIDE};
use ennui::later::{set, set_if_new};
use ennui::prelude::{Entity, Glance, Later, Mut, Res, View, each, each_mut};
use ennui::storage::query;
use ennui_platform::prelude::{Input, MouseButton};
use ennui_ui::prelude::{
    Float, Hidden, Hosted, Hosts, Panel, Press, Rect, Theme, Tone, pointer_in, sweep,
};

use nalgebra_glm::Vec2;

pub(crate) fn lay_boards(rows: Glance, mut later: Later, look: Res<Theme>) {
    let gap = BAR;
    let least = Vec2::new(LEAST_WIDE, LEAST_TALL);
    for (board, (held, rect)) in query::<(&Board, &Rect)>(&rows) {
        let mut laid = held.0.laid.clone();
        lay(&held.0, &mut laid, *rect, (gap, least));
        if laid == held.0.laid {
            continue;
        }
        let tiles = Tiles {
            laid,
            ..held.0.clone()
        };
        for (pane, content) in panes_of(&tiles) {
            let room = tiles.laid.get(pane).copied().unwrap_or_default();
            let shown = pane_shown(&tiles, pane) && room.size.x > 0.0;
            set_if_new(&mut later, content, Hidden(!shown));
            if !shown {
                continue;
            }
            resize(&rows, &mut later, content, room.size);
            set_if_new(&mut later, content, Float(corner(room)));
        }
        make_sashes(&rows, &mut later, &look, board, (&tiles, gap, least));
        make_strips(&rows, &mut later, &look, board, &tiles);
        sweep(&mut later, board);
        set(&mut later, board, Board(tiles));
    }
}

pub(crate) fn drag_sashes(
    mut boards: Mut<(Board,), (Option<&Hosted>,)>,
    sashes: View<(&Sash, &Press)>,
    hosts: Res<Hosts>,
    input: Res<Input>,
) {
    if input.pointer_motion == [0.0, 0.0] {
        return;
    }
    let gap = BAR;
    let least = Vec2::new(LEAST_WIDE, LEAST_TALL);
    let held: Vec<Sash> = each(&sashes)
        .filter(|(_, (_, press))| press.0)
        .map(|(_, (sash, _))| *sash)
        .collect();
    if held.is_empty() {
        return;
    }
    each_mut(&mut boards, |entity, (mut board,), (hosted,)| {
        let at = pointer_in(&hosts, hosted);
        for sash in held.iter().filter(|sash| sash.board == entity) {
            let mut tiles = board.0.clone();
            let rect = tiles.laid.get(sash.tile).copied().unwrap_or_default();
            if let Some(Some(Tile::Split {
                down: way, share, ..
            })) = tiles.held.get_mut(sash.tile)
            {
                let along = match *way {
                    true => {
                        (rect.center.y + rect.size.y * 0.5 - at.y - gap * 0.5)
                            / (rect.size.y - gap).max(f32::EPSILON)
                    }
                    false => {
                        (at.x - rect.center.x + rect.size.x * 0.5 - gap * 0.5)
                            / (rect.size.x - gap).max(f32::EPSILON)
                    }
                };
                let (low, high) = share_range(rect, *way, gap, least);
                let wanted = along.clamp(low, high);
                if (wanted - *share).abs() > f32::EPSILON {
                    *share = wanted;
                    *board = Board(tiles);
                }
            }
        }
    });
}

pub(crate) fn drag_panes(
    mut boards: Dragging<'_, '_>,
    mut marks: Mut<(Hidden, Panel, Float, Tone)>,
    leaves: View<(&Leafed, &Press)>,
    hosts: Res<Hosts>,
    input: Res<Input>,
    look: Res<Theme>,
) {
    let down = input.buttons_held.contains(&MouseButton::Left);
    let began = input.buttons_pressed.contains(&MouseButton::Left);
    let taken: Vec<(Entity, usize)> = each(&leaves)
        .filter(|(_, (_, press))| press.0)
        .map(|(_, (leafed, _))| (leafed.board, leafed.tile))
        .collect();
    let mut shown: Vec<(Entity, Shown)> = Vec::new();
    each_mut(
        &mut boards,
        |board, (mut tiles, mut dragged), (zone, hosted)| {
            let at = pointer_in(&hosts, hosted);
            let grabbed = taken
                .iter()
                .find(|(held, _)| began && *held == board)
                .map(|(_, pane)| *pane);
            let wanted = drag_board(&mut tiles, &mut dragged, grabbed, at, down);
            if let Some(wanted) = wanted
                && let Some(mark) = zone.map(|held| held.0)
            {
                shown.push((mark, wanted));
            }
        },
    );
    show_zones(&mut marks, shown, look.chosen);
}
