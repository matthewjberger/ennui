use super::press::selectable;
use super::text::small;
use crate::components::{Band, Menu, Opened, Pick};
use crate::queries::shape::list_frame;
use crate::theme::{LIST_GAP, MENU_DROP, MENU_ROWS, MENU_SOFT, MENU_WIDE};
use ennui::later::{attach, change, within};
use ennui::prelude::{Edits, Entity, Later};
use ennui::storage::{get, is_alive};
use ennui_scene::prelude::despawn_trees;
use ennui_ui::prelude::{
    Dye, Float, Frame, Hidden, Inside, Line, Span, Tether, Theme, frame, panel, scroll, separator,
    under,
};

use nalgebra_glm::Vec2;

pub fn menu(
    later: &mut Later,
    look: &Theme,
    over: Entity,
    target: Entity,
    options: &[&str],
) -> Entity {
    change(later, move |storage| {
        if let Some(old) = get::<Menu>(&*storage, target).map(|held| held.0)
            && is_alive(&*storage, old)
        {
            within(storage, |later| despawn_trees(later, vec![old]));
        }
    });
    let list = frame(
        later,
        list_frame(look, MENU_WIDE).shade(
            Dye::Shade,
            [0.0, -(look.drop * MENU_DROP), look.soft * MENU_SOFT, 0.0],
        ),
    );
    under(later, over, list);
    attach(
        later,
        list,
        (Float(Vec2::zeros()), Inside, Hidden(true), Tether(target)),
    );
    let held = match options.len() > MENU_ROWS {
        true => scroll(
            later,
            look,
            list,
            Frame::new(look)
                .wide(Span::Fill(1.0))
                .tall(Span::Fixed(look.row * MENU_ROWS as f32))
                .along(Line::Start)
                .bare()
                .pad(0.0)
                .gap(LIST_GAP),
        ),
        false => list,
    };
    let mut index = 0;
    for option in options.iter() {
        if option.is_empty() {
            separator(later, look, held);
            continue;
        }
        let (text, hint) = option.split_once('\t').unwrap_or((option, ""));
        let row = selectable(later, look, held, text, false);
        if !hint.is_empty() {
            panel(later, row, Frame::new(look).wide(Span::Fill(1.0)).bare());
            small(later, look, row, hint);
        }
        attach(later, row, (Pick(index), Band(target)));
        index += 1;
    }
    attach(later, target, (Menu(list), Opened(false)));
    list
}

pub fn shut_menu(edits: &mut Edits, target: Entity) {
    change(edits, move |storage| {
        let Some(list) = get::<Menu>(&*storage, target).map(|held| held.0) else {
            return;
        };
        ennui::storage::set(&mut *storage, target, Opened(false));
        ennui::storage::set(&mut *storage, list, Hidden(true));
    });
}

pub fn open_menu(edits: &mut Edits, target: Entity, at: Vec2) {
    change(edits, move |storage| {
        let Some(list) = get::<Menu>(&*storage, target).map(|held| held.0) else {
            return;
        };
        ennui::storage::set(&mut *storage, target, Opened(true));
        ennui::storage::attach(&mut *storage, list, (Hidden(false), Float(at)));
    });
}
