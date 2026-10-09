use crate::components::Sketch;
use crate::data::PaintedSketches;
use ennui::prelude::{Peek, Res, ResMut, View, each, peek};
use ennui_quads::prelude::{Quad, Quads};
use ennui_scene::prelude::ChildOf;
use ennui_text::prelude::{Cut, Deep};
use ennui_ui::prelude::{Hidden, Hosted, Hosts, Rect, staged, veiled};

pub(crate) fn paint_sketches(
    drawn: View<(&Sketch, &Rect, &Hosted)>,
    cuts: Peek<Cut>,
    deeps: Peek<Deep>,
    hidden: Peek<Hidden>,
    parents: Peek<ChildOf>,
    hosts: Res<Hosts>,
    mut quads: ResMut<Quads<PaintedSketches>>,
) {
    quads.list.clear();
    for (entity, (sketch, rect, hosted)) in each(&drawn) {
        if sketch.0.is_empty() || veiled(&hidden, &parents, entity) {
            continue;
        }
        let Some(hosting) = hosts.list.iter().find(|held| held.host == hosted.0) else {
            continue;
        };
        let clip = peek(&cuts, entity).map_or(
            [
                rect.center.x,
                rect.center.y,
                rect.size.x * 0.5,
                rect.size.y * 0.5,
            ],
            |held| held.0,
        );
        let depth = peek(&deeps, entity).map_or(0.0, |held| held.0) + 0.5;
        for held in sketch.0.iter() {
            let quad = Quad {
                center: [
                    rect.center.x + held.center[0],
                    rect.center.y + held.center[1],
                ],
                clip,
                depth,
                ..*held
            };
            quads.list.push(staged(hosting, quad));
        }
    }
}
