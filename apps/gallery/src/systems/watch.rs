use crate::components::{Named, Noted};
use crate::queries::controls::{palette_rows, reading};
use crate::queries::dock::{pointed_pane, tree_of};
use crate::resources::Shown;
use crate::{data, theme};
use ennui::prelude::{Edits, Glance, Later, Peek, Res, ResMut, View, each, peek};
use ennui::storage::{get, query};
use ennui_platform::prelude::{Claimed, Input, KeyCode, Time, Viewport};
use ennui_text::prelude::write_label;
use ennui_ui::prelude::screen_from_pointer;
use ennui_ui::prelude::{Click, Hidden, Hover, Theme, clicked};

use ennui_ui_controls::prelude::{
    Note, Opened, Ran, Slide, Tray, chords, menu_choice, offer, open_menu, raise, shut_menu, toast,
};
use ennui_ui_controls::prelude::{Progress, clear_progress, set_progress, slide_progress};
use ennui_ui_dock::prelude::Board;

pub(crate) fn read_controls(
    seen: Glance,
    mut later: Later,
    look: Res<Theme>,
    mut held: ResMut<Shown>,
    mut tray: ResMut<Tray>,
) {
    if clicked(&seen, held.press) {
        held.count += 1;
    }
    if let Some(target) = held.menu_target {
        if held.menu_asked && !held.menu_opened {
            open_menu(&mut later, target, theme::MENU_AT);
            held.menu_opened = true;
        }
        if let Some(choice) = menu_choice(&seen, target) {
            let said = data::MENU_NAMES.get(choice).copied().unwrap_or_default();
            write_label(&mut later, held.menu_said, format!("{said} FROM THE MENU"));
        }
    }
    let wanted = reading(&seen, &held);
    write_label(&mut later, held.reading, wanted);

    let asked: Vec<usize> = query::<(&Noted, &Click)>(&seen)
        .filter(|(_, (_, click))| click.0)
        .map(|(_, (noted, _))| noted.0)
        .collect();
    for place in asked {
        let (_, text, kind) = data::NOTES[place];
        toast(
            &seen,
            &mut later,
            &look,
            &mut tray,
            text,
            kind,
            theme::TOAST_LIFE,
        );
    }
    if clicked(&seen, held.raise_tell)
        && let Some(over) = held.tell
    {
        raise(&seen, &mut later, over);
    }
    if clicked(&seen, held.raise_ask)
        && let Some(over) = held.ask
    {
        raise(&seen, &mut later, over);
    }
}

pub(crate) fn run_palette(
    seen: Glance,
    mut later: Later,
    input: Res<Input>,
    claimed: Res<Claimed>,
    look: Res<Theme>,
    time: Res<Time>,
    mut held: ResMut<Shown>,
    mut tray: ResMut<Tray>,
) {
    let Some(over) = held.palette else {
        return;
    };
    let (control, _) = chords(&input);
    let keyed = control && input.pressed.contains(&KeyCode::KeyK) && claimed.keys.is_empty();
    let shut = get::<Hidden>(&seen, over).is_some_and(|hidden| hidden.0);
    if shut && (keyed || clicked(&seen, held.raise_palette)) {
        raise(&seen, &mut later, over);
    }
    let said = get::<Ran>(&seen, over)
        .and_then(|ran| ran.0)
        .and_then(|(payload, alternate)| {
            let chosen = held.offers.iter().find(|offer| offer.payload == payload)?;
            let title = chosen.title.to_uppercase();
            Some(match alternate {
                true => format!("{title}: {}", chosen.also),
                false => format!("RAN {title}"),
            })
        });
    let tick = (time.since_start / theme::PALETTE_SECONDS) as usize;
    if held.offered != Some((over, tick)) {
        held.offered = Some((over, tick));
        held.offers = palette_rows(tick);
        offer(&mut later, over, held.offers.clone());
    }
    let Some(said) = said else {
        return;
    };
    toast(
        &seen,
        &mut later,
        &look,
        &mut tray,
        &said,
        Note::Good,
        theme::TOAST_LIFE,
    );
}

pub(crate) fn name_icon(cells: View<(&Named, &Hover)>, mut edits: Edits, held: Res<Shown>) {
    let pointed = each(&cells)
        .find(|(_, (_, hover))| hover.0)
        .map(|(_, (named, _))| named.0);
    let said = match pointed {
        Some(name) => format!("POINTING AT {name}"),
        None => String::from("POINT AT AN ICON"),
    };
    write_label(&mut edits, held.icon_said, said);
}

pub(crate) fn shut_by_key(
    opened: Peek<Opened>,
    mut edits: Edits,
    input: Res<Input>,
    held: Res<Shown>,
) {
    let Some(target) = held.menu_target else {
        return;
    };
    let open = peek(&opened, target).is_some_and(|opened| opened.0);
    if open && input.pressed.contains(&KeyCode::Escape) {
        shut_menu(&mut edits, target);
        write_label(&mut edits, held.menu_said, String::from("SHUT BY CODE"));
    }
}

pub(crate) fn watch_dock(
    boards: Peek<Board>,
    mut edits: Edits,
    input: Res<Input>,
    viewport: Res<Viewport>,
    held: Res<Shown>,
) {
    let Some(board) = held.deck.and_then(|deck| peek(&boards, deck)) else {
        return;
    };
    let tiles = &board.0;
    let at = screen_from_pointer(viewport.width, viewport.height, input.pointer);
    write_label(&mut edits, held.dock_tree, tree_of(tiles, tiles.root));
    write_label(&mut edits, held.dock_pointer, pointed_pane(tiles, at));
}

pub(crate) fn drive_progress(
    slides: Peek<Slide>,
    clicks: Peek<Click>,
    mut held: ResMut<Shown>,
    mut progress: ResMut<Progress>,
) {
    let share = held
        .progress_slide
        .and_then(|entity| peek(&slides, entity))
        .map_or(0.0, |slide| slide.0);
    if share != held.progress_last {
        held.progress_last = share;
        set_progress(&mut progress, share);
    }
    if clicked(&clicks, held.progress_sliding) {
        slide_progress(&mut progress);
    }
    if clicked(&clicks, held.progress_clear) {
        clear_progress(&mut progress);
    }
}
