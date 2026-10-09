use crate::commands::atlas::{marked, restart};
use crate::commands::shape::laid;
use crate::data::{Align, WARMED};
use crate::resources::Glyphs;
use crate::theme::WARMED_EACH_FRAME;
use ennui::prelude::ResMut;
use ennui_render::resources::Images;

pub(crate) fn clear_full_sheet(mut glyphs: ResMut<Glyphs>) {
    if glyphs.full {
        restart(&mut glyphs);
    }
}

pub(crate) fn hold_sheet(mut glyphs: ResMut<Glyphs>, mut images: ResMut<Images>) {
    if !glyphs.dirty {
        return;
    }
    glyphs.dirty = false;
    let slot = glyphs.slot.0;
    if let Some(held) = images.list.get_mut(slot) {
        held.pixels.clone_from(&glyphs.sheet.pixels);
        images.dirty.push(slot);
    }
}

pub(crate) fn open_sheet(mut glyphs: ResMut<Glyphs>, mut images: ResMut<Images>) {
    glyphs.slot = ennui_render::commands::assets::insert_image(&mut images, glyphs.sheet.clone());
    let places = laid(&mut glyphs, "", WARMED, 0.0, Align::Start);
    glyphs.waiting = places.iter().rev().map(|(_, _, key)| *key).collect();
}

pub(crate) fn warm_glyphs(mut glyphs: ResMut<Glyphs>) {
    for _ in 0..WARMED_EACH_FRAME {
        let Some(key) = glyphs.waiting.pop() else {
            return;
        };
        marked(&mut glyphs, key);
    }
}
