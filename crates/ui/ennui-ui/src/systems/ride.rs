use crate::commands::renew::renew;
use crate::components::{Float, Hidden, Hosted, Panel, Press, Reach, Rect, Scroll, Thumb};
use crate::data::Span;
use crate::queries::ride::groove;
use crate::queries::theme::{theme_of, worn_theme};
use crate::resources::{Hosts, Theme};
use crate::theme::THUMB_ROOM;
use ennui::prelude::{Mut, Peek, Res, View, each, each_mut, peek, peek_mut};
use ennui_platform::prelude::Input;
use nalgebra_glm::Vec2;

pub(crate) fn ride_thumbs(
    bars: View<(&Thumb, &Scroll, &Reach, &Rect, Option<&Hosted>)>,
    mut thumbs: Mut<(Hidden, Panel, Float)>,
    themes: Peek<Theme>,
    theme: Res<Theme>,
    hosts: Res<Hosts>,
) {
    for (_, (thumb, scroll, reach, rect, hosted)) in each(&bars) {
        let Some((mut hidden, mut panel, mut float)) = peek_mut(&mut thumbs, thumb.0) else {
            continue;
        };
        let (offset, reach) = (scroll.0, reach.0);
        renew(&mut hidden, Hidden(reach <= 0.0));
        if reach <= 0.0 {
            continue;
        }
        let worn = worn_theme(&themes, &theme, &hosts, hosted);
        let (inset, room, bar) = groove(worn, rect.size.y, reach);
        let travel = (room - bar) * (offset / reach).clamp(0.0, 1.0);
        if panel.tall != Span::Fixed(bar) {
            panel.tall = Span::Fixed(bar);
        }
        renew(
            &mut float,
            Float(Vec2::new(
                rect.center.x + rect.size.x * 0.5 - inset - worn.bar_wide,
                rect.center.y + rect.size.y * 0.5 - inset - travel,
            )),
        );
    }
}

pub(crate) fn drag_thumbs(
    mut bars: Mut<(Scroll,), (&Thumb, &Reach, &Rect, Option<&Hosted>)>,
    presses: Peek<Press>,
    themes: Peek<Theme>,
    theme: Res<Theme>,
    hosts: Res<Hosts>,
    input: Res<Input>,
) {
    let motion = input.pointer_motion[1];
    if motion == 0.0 {
        return;
    }
    each_mut(
        &mut bars,
        |_, (mut scroll,), (thumb, reach, rect, hosted)| {
            if !peek(&presses, thumb.0).is_some_and(|press| press.0) {
                return;
            }
            let (offset, reach) = (scroll.0, reach.0);
            if reach <= 0.0 {
                return;
            }
            let Some(hosting) =
                hosted.and_then(|hosted| hosts.list.iter().find(|held| held.host == hosted.0))
            else {
                return;
            };
            let worn = theme_of(&themes, &theme, hosting.theme);
            let (_, room, bar) = groove(worn, rect.size.y, reach);
            let travel = (room - bar).max(THUMB_ROOM);
            let moved = motion / hosting.pixels.max(f32::EPSILON) * reach / travel;
            renew(&mut scroll, Scroll((offset + moved).clamp(0.0, reach)));
        },
    );
}
