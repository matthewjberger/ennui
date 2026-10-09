use crate::commands::edit::{
    edit_ahead, edit_all, edit_back, edit_clear, edit_end, edit_home, edit_left, edit_put,
    edit_right, edit_taken, edit_word_left, edit_word_right,
};
use crate::components::Entered;
use crate::data::Edit;
use crate::queries::keys::struck;
use crate::theme::LINE_STEP;
use ennui::prelude::{Mut, each_mut};
use ennui_platform::prelude::{Input, KeyCode, write_clipboard};
use ennui_text::prelude::{Glyphs, index_near, spot_at};
use nalgebra_glm::Vec2;

pub(crate) fn step_line(
    glyphs: &mut Glyphs,
    (tall, room, family): (f32, f32, &str),
    edit: &mut Edit,
    way: f32,
    keep: bool,
) {
    if room <= 0.0 {
        return;
    }
    let spot = spot_at(glyphs, family, &edit.text, tall, room, edit.at);
    let wanted = Vec2::new(spot.x, spot.y + way * tall * LINE_STEP);
    edit.at = index_near(glyphs, family, &edit.text, tall, room, wanted);
    if !keep {
        edit.mark = edit.at;
    }
}

pub(crate) fn clear_entered(entered: &mut Mut<(Entered,)>) {
    each_mut(entered, |_, (mut held,), ()| {
        if held.0 {
            *held = Entered(false);
        }
    });
}

pub(crate) fn press_keys(wanted: &mut Edit, input: &Input, control: bool, keep: bool, room: usize) {
    if struck(input, KeyCode::Backspace) {
        edit_back(wanted);
    }
    if struck(input, KeyCode::Delete) {
        edit_ahead(wanted);
    }
    if struck(input, KeyCode::ArrowLeft) {
        match control {
            true => edit_word_left(wanted, keep),
            false => edit_left(wanted, keep),
        }
    }
    if struck(input, KeyCode::ArrowRight) {
        match control {
            true => edit_word_right(wanted, keep),
            false => edit_right(wanted, keep),
        }
    }
    if struck(input, KeyCode::Home) {
        edit_home(wanted, keep);
    }
    if struck(input, KeyCode::End) {
        edit_end(wanted, keep);
    }
    if control && input.pressed.contains(&KeyCode::KeyA) {
        edit_all(wanted);
    }
    let cut = control && input.pressed.contains(&KeyCode::KeyX);
    if (control && input.pressed.contains(&KeyCode::KeyC)) || cut {
        write_clipboard(&edit_taken(wanted));
    }
    if cut {
        edit_clear(wanted);
    }
    if let Some(text) = input.pasted.as_ref() {
        let room_left = room.saturating_sub(wanted.text.chars().count());
        let trimmed: String = text
            .chars()
            .filter(|held| *held != '\n')
            .take(room_left)
            .collect();
        edit_put(wanted, &trimmed);
    }
    if input.pressed.contains(&KeyCode::Enter) && room == usize::MAX {
        edit_put(wanted, "\n");
    }
}

pub(crate) fn type_letters(wanted: &mut Edit, letters: &str, control: bool, room: usize) {
    for letter in letters.chars() {
        if wanted.text.chars().count() >= room || control || letter.is_control() {
            break;
        }
        edit_put(wanted, &letter.to_string());
    }
}
