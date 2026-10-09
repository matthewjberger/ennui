use crate::commands::hint::{hide_hint, raise_hint};
use crate::components::{Hint, Opened, Tip};
use crate::resources::Hinting;
use crate::theme::HINT_DELAY;
use ennui::later::set;
use ennui::prelude::{Later, Peek, Res, ResMut, View, each, peek};
use ennui_platform::prelude::Time;
use ennui_ui::prelude::{Hidden, Hover, Press, Rect, Theme};

pub(crate) fn show_tips(
    tips: View<(&Tip, &Hover, &Press)>,
    hints: View<(&Hint, &Hover, &Press)>,
    menus: View<(&Opened,)>,
    rects: Peek<Rect>,
    mut hinting: ResMut<Hinting>,
    mut later: Later,
    look: Res<Theme>,
    time: Res<Time>,
) {
    let tipped = each(&tips)
        .filter(|(_, (_, hover, _))| hover.0)
        .map(|(entity, (tip, _, press))| (entity, press.0, Some(tip.0)))
        .last();
    let hinted = each(&hints)
        .filter(|(_, (_, hover, _))| hover.0)
        .map(|(entity, (_, _, press))| (entity, press.0, None))
        .last();
    let hovered = hinted.or(tipped);
    let over = hovered.map(|(entity, _, _)| entity);
    let pressed = hovered.is_some_and(|(_, press, _)| press);
    let menu_open = each(&menus).any(|(_, (opened,))| opened.0);
    if over != hinting.over || pressed || menu_open {
        hide_hint(&mut later, &mut hinting);
        hinting.quiet = pressed;
        hinting.over = over;
        hinting.since = 0.0;
        return;
    }
    let Some((entity, _, tip)) = hovered else {
        return;
    };
    if hinting.quiet || hinting.card.is_some() {
        return;
    }
    hinting.since += time.since_last_frame;
    if hinting.since < HINT_DELAY {
        return;
    }
    hinting.card = Some(match tip {
        Some(card) => {
            set(&mut later, card, Hidden(false));
            card
        }
        None => {
            let kept = hinting.sheet.filter(|held| peek(&rects, *held).is_some());
            let text = each(&hints)
                .find(|(held, _)| *held == entity)
                .map_or("", |(_, (hint, _, _))| hint.0.as_str());
            raise_hint(&mut later, &mut hinting, &look, kept, (entity, text))
        }
    });
}
