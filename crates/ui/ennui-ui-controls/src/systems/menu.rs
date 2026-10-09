use crate::components::{Band, Menu, Opened, Pick};
use crate::data::Menus;
use ennui::prelude::{Entity, Mut, Peek, Res, View, each, each_mut, peek};
use ennui_platform::prelude::{Input, MouseButton};
use ennui_ui::prelude::{Click, Float, Hidden, Hosts, Rect, inside_of, pointer_in, renew_each};
use nalgebra_glm::Vec2;

pub(crate) fn open_menus(
    mut menus: Menus,
    mut hidden: Mut<(Hidden,)>,
    mut floats: Mut<(Float,)>,
    rects: Peek<Rect>,
    hosts: Res<Hosts>,
    input: Res<Input>,
) {
    let asked = input.buttons_pressed.contains(&MouseButton::Right);
    let shut = input.buttons_pressed.contains(&MouseButton::Left);
    if !asked && !shut {
        return;
    }
    let mut lists: Vec<(Entity, bool, Vec2)> = Vec::new();
    each_mut(
        &mut menus,
        |_, (mut opened,), (menu, rect, hosted, hover)| {
            let at = pointer_in(&hosts, hosted);
            let list = menu.0;
            let aimed = hover.map_or_else(|| inside_of(at, rect), |hover| hover.0);
            if asked && aimed {
                *opened = Opened(true);
                lists.push((list, true, at));
                return;
            }
            let over_list = peek(&rects, list).is_some_and(|list| inside_of(at, list));
            if opened.0 && (asked || (shut && !over_list)) {
                *opened = Opened(false);
                lists.push((list, false, at));
            }
        },
    );
    renew_each(
        &mut hidden,
        lists.iter().map(|(list, open, _)| (*list, Hidden(!open))),
    );
    let opened = lists.into_iter().filter(|(_, open, _)| *open);
    renew_each(&mut floats, opened.map(|(list, _, at)| (list, Float(at))));
}

pub(crate) fn shut_menus(
    picks: View<(&Pick, &Band, &Click)>,
    menus: Peek<Menu>,
    mut opened: Mut<(Opened,)>,
    mut hidden: Mut<(Hidden,)>,
) {
    let taken: Vec<(Entity, Entity)> = each(&picks)
        .filter(|(_, (_, _, click))| click.0)
        .filter_map(|(_, (_, band, _))| Some((band.0, peek(&menus, band.0)?.0)))
        .collect();
    renew_each(
        &mut opened,
        taken.iter().map(|(owner, _)| (*owner, Opened(false))),
    );
    renew_each(
        &mut hidden,
        taken.into_iter().map(|(_, list)| (list, Hidden(true))),
    );
}
