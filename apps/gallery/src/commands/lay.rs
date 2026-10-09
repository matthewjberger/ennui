use crate::components::{Named, Noted};
use crate::resources::Shown;
use crate::{data, theme};
use ennui::later::set;
use ennui::prelude::{Entity, Later};
use ennui_render::data::TextureId;
use ennui_ui::prelude::{
    Dye, Frame, Lay, Line, Lit, Span, Theme, ink, label, panel, relay, screen, scroll, separator,
    sheet, spacing, touched, wrapped,
};
use gallery_widgets::prelude::{
    Streamed, almanac, breadcrumb, canvas, grid, icon_button, multi, range, runs, spinner, split,
};

use ennui_ui_controls::prelude::{
    Band, Chose, Close, PALETTE_SCOPES, Sprig, Tab, area, bar, branch, button, checkbox,
    collapsing, confirm, entry, field, filling, ledger, listed, menu, modal, page, palette, picker,
    radio, scrub, selectable, slider, small, stat, swatch, tabs, toggle, tooltip, twig,
};
use ennui_ui_dock::prelude::{board, pane};
use ennui_ui_icons::prelude::icon;

pub(crate) fn build_all(
    later: &mut Later,
    look: &Theme,
    shown: &mut Shown,
    badge: TextureId,
    opening: usize,
) {
    let root = screen(later, look);
    let over = sheet(later, look, theme::SHEET_ORDER);
    let (rail, stack) = lay_board(later, look, root);
    lay_top(later, look, shown, stack, over);
    let holder = panel(later, stack, filling(look).gap(0.0));
    set(later, holder, Chose(opening));
    shown.tabs = Some(holder);
    lay_nav(later, look, rail, holder, opening);
    lay_buttons(later, look, holder, over, badge, shown);
    lay_inputs(later, look, holder, over, shown);
    lay_containers(later, look, holder, over, shown);
    lay_data(later, look, holder, shown);
    lay_out(later, look, holder, over, shown);
    lay_text(later, look, holder, shown);
    lay_sheet(later, look, holder, over);
    lay_dock(later, look, holder, shown);
}

fn lay_board(later: &mut Later, look: &Theme, root: Entity) -> (Entity, Entity) {
    let board = panel(
        later,
        root,
        Frame::new(look)
            .flow(Lay::Row)
            .wide(Span::Fill(1.0))
            .tall(Span::Fill(1.0))
            .role(Dye::Ground)
            .round(0.0)
            .pad(0.0)
            .gap(0.0),
    );
    let rail = panel(
        later,
        board,
        Frame::new(look)
            .wide(Span::Fixed(theme::RAIL))
            .tall(Span::Fill(1.0))
            .along(Line::Start)
            .role(Dye::Panel)
            .round(0.0)
            .pad(look.pad * theme::RAIL_PAD)
            .gap(theme::RAIL_GAP),
    );
    label(later, look, rail, "GALLERY", look.heading);
    spacing(later, look, rail, theme::RAIL_GAP);
    let stack = panel(later, board, filling(look).gap(0.0));
    (rail, stack)
}

fn lay_top(later: &mut Later, look: &Theme, shown: &mut Shown, stack: Entity, over: Entity) {
    let top = panel(
        later,
        stack,
        Frame::row(look)
            .tall(Span::Fixed(theme::TOP_BAR))
            .role(Dye::Header)
            .round(0.0)
            .pad(theme::PAGE_PAD),
    );
    shown.reading = Some(label(later, look, top, "READY", look.text));
    spacing(later, look, top, 0.0);
    let wall = panel(
        later,
        top,
        Frame::new(look)
            .flow(Lay::Row)
            .wide(Span::Fill(1.0))
            .tall(Span::Fill(1.0))
            .along(Line::End)
            .bare()
            .pad(0.0)
            .gap(look.gap * 0.5),
    );
    let names: Vec<&str> = data::THEMES.iter().map(|(name, _)| *name).collect();
    let nook = panel(
        later,
        wall,
        Frame::new(look)
            .wide(Span::Fixed(theme::PICKER))
            .tall(Span::Fill(1.0))
            .bare()
            .pad(0.0)
            .gap(0.0),
    );
    shown.theme_pick = Some(listed(
        later,
        look,
        [nook, over],
        &names,
        shown.theme,
        false,
    ));
}

fn lay_nav(later: &mut Later, look: &Theme, rail: Entity, holder: Entity, opening: usize) {
    for (index, (name, note, glyph)) in data::SCREENS.iter().enumerate() {
        let item = panel(
            later,
            rail,
            Frame::new(look)
                .along(Line::Start)
                .wide(Span::Fill(1.0))
                .tall(Span::Fixed(theme::NAV_ROW))
                .roles(Dye::Header, Dye::Accent)
                .pad(look.pad * theme::NAV_PAD)
                .gap(0.0),
        );
        set(later, item, Lit(index == opening));
        let line = panel(
            later,
            item,
            Frame::row(look).bare().pad(0.0).gap(look.gap * 0.5),
        );
        icon(later, look, line, *glyph, look.text);
        label(later, look, line, name, look.text);
        let note = small(later, look, item, note);
        ink(later, note, Dye::Faint, Dye::Ink);
        set(later, note, Lit(index == opening));
        set(later, item, Tab(index));
        set(later, item, Band(holder));
        touched(later, item);
    }
}

fn lay_buttons(
    later: &mut Later,
    look: &Theme,
    holder: Entity,
    over: Entity,
    badge: TextureId,
    shown: &mut Shown,
) {
    let buttons = page_of(later, look, holder, 0);
    let block = block_of(later, look, buttons, "BUTTONS");
    shown.press = Some(button(later, look, block, "PRESS ME"));
    let icons = panel(later, block, Frame::row(look).bare().pad(0.0).gap(look.gap));
    let marked = icon_button(later, look, icons, badge, look.row);
    tooltip(later, look, over, marked, "AN ICON BUTTON");
    icon_button(later, look, icons, badge, look.row);
    separator(later, look, block);
    shown.tick = Some(checkbox(later, look, block, "NOTIFICATIONS", true));
    shown.switch = Some(toggle(later, look, block, "DARK MODE", false));
    separator(later, look, block);
    radio(later, look, block, block, "WINDOWED", true);
    radio(later, look, block, block, "FULL SCREEN", false);

    let block = block_of(later, look, buttons, "PROGRESS");
    breadcrumb(later, look, block, &data::CRUMBS);
    spinner(later, look, block);
    for (name, value) in data::STATS.iter() {
        stat(later, look, block, name, value);
    }

    let block = block_of(later, look, buttons, "WINDOW PROGRESS");
    small(later, look, block, data::PROGRESS_NOTE);
    shown.progress_slide = Some(slider(later, look, block, 0.0));
    shown.progress_sliding = Some(button(later, look, block, "SET SLIDING"));
    shown.progress_clear = Some(button(later, look, block, "CLEAR"));
    shown.icon_said = Some(lay_icons(later, look, buttons, over));
}

fn lay_icons(later: &mut Later, look: &Theme, buttons: Entity, over: Entity) -> Entity {
    let block = block_of(later, look, buttons, "ICONS");
    let said = label(later, look, block, "POINT AT AN ICON", look.text);
    for chunk in data::SHOWN_ICONS.chunks(theme::ICONS_ACROSS) {
        let shelf = panel(later, block, Frame::row(look).bare().pad(0.0).gap(look.gap));
        for (glyph, name) in chunk {
            let cell = panel(
                later,
                shelf,
                Frame::new(look)
                    .wide(Span::Fixed(theme::ICON_CELL))
                    .tall(Span::Fixed(theme::ICON_CELL))
                    .role(Dye::Header)
                    .pad(0.0),
            );
            icon(later, look, cell, *glyph, look.heading);
            touched(later, cell);
            tooltip(later, look, over, cell, name);
            set(later, cell, Named(name));
        }
    }
    said
}

fn lay_inputs(later: &mut Later, look: &Theme, holder: Entity, over: Entity, shown: &mut Shown) {
    let inputs = page_of(later, look, holder, 1);
    let block = block_of(later, look, inputs, "INPUTS");
    shown.slide = Some(slider(later, look, block, theme::SLIDER_START));
    bar(later, look, block, theme::BAR_START);
    let (low, high) = theme::RANGE_START;
    range(later, look, block, low, high);
    shown.name = Some(field(
        later,
        look,
        block,
        "fox",
        "name",
        theme::FIELD_LETTERS,
    ));
    field(later, look, block, "", "type a word", theme::FIELD_LETTERS);

    let pickers = page_of(later, look, holder, 2);
    let block = block_of(later, look, pickers, "PICKERS");
    shown.quality = Some(listed(later, look, [block, over], &data::QUALITY, 2, false));
    listed(later, look, [block, over], &data::STOCK_NAMES, 0, true);
    multi(later, look, block, &data::STOCK_NAMES);
    let inner = tabs(later, look, block, &data::MODES, (0, data::MODES.len()));
    let first = page(later, look, inner, 0);
    label(later, look, first, "THE FIRST MODE", look.text);
    let second = page(later, look, inner, 1);
    label(later, look, second, "THE SECOND MODE", look.text);
}

fn lay_containers(
    later: &mut Later,
    look: &Theme,
    holder: Entity,
    over: Entity,
    shown: &mut Shown,
) {
    let holders = page_of(later, look, holder, 3);
    let block = block_of(later, look, holders, "CONTAINERS");
    for (place, name) in data::SECTIONS.iter().enumerate() {
        let body = collapsing(later, look, block, name, place == 0, Some(data::MARKS));
        for line in 0..3 {
            label(
                later,
                look,
                body,
                &format!("{name} LINE {}", line + 1),
                look.text,
            );
        }
    }
    let list = scroll(
        later,
        look,
        block,
        Frame::new(look)
            .wide(Span::Fill(1.0))
            .tall(Span::Fixed(theme::LIST_TALL))
            .along(Line::Start)
            .role(Dye::Input)
            .pad(look.pad * 0.5)
            .gap(2.0),
    );
    for place in 0..data::ROWS {
        selectable(later, look, list, &format!("ROW {}", place + 1), place == 2);
    }
    let line = panel(
        later,
        block,
        Frame::row(look).bare().pad(0.0).gap(look.gap * 0.5),
    );
    for (place, (name, _, _)) in data::NOTES.iter().enumerate() {
        let pressed = button(later, look, line, name);
        set(later, pressed, Noted(place));
    }
    shown.raise_tell = Some(button(later, look, block, "SHOW A NOTICE"));
    shown.raise_ask = Some(button(later, look, block, "ASK TO QUIT"));
    let searching = button(later, look, block, "SEARCH AND RUN");
    tooltip(later, look, over, searching, theme::PALETTE_TIP);
    shown.raise_palette = Some(searching);

    let scrim = || Frame::new(look).fill(theme::DIALOG_SCRIM).pad(0.0);
    let dialog = || {
        Frame::new(look)
            .wide(Span::Fixed(look.row * theme::DIALOG_ROWS))
            .along(Line::Middle)
            .pad(look.pad)
    };
    let (tell, card, _) = modal(
        later,
        look,
        theme::TELL_ORDER,
        scrim(),
        dialog(),
        "A NOTICE",
    );
    label(later, look, card, "THE WORLD WAS SAVED", look.text);
    let close = button(later, look, card, "CLOSE");
    set(later, tell, Close(close));
    shown.tell = Some(tell);
    let (ask, card, _) = modal(later, look, theme::ASK_ORDER, scrim(), dialog(), "LEAVE");
    shown.ask = Some(confirm(later, look, ask, card, "GIVE UP THIS RUN?"));
    shown.palette = Some(palette(later, look, theme::PALETTE_ORDER, &PALETTE_SCOPES));
}

fn lay_sheet(later: &mut Later, look: &Theme, holder: Entity, over: Entity) {
    let page = page_of(later, look, holder, 7);
    let block = block_of(later, look, page, "AN INSPECTOR");
    let held = ledger(later, look, block, theme::LABEL_SPAN);
    let line = entry(later, look, held, "NAME");
    field(later, look, line, "anvil", "a name", theme::FIELD_LETTERS);
    let line = entry(later, look, held, "WEIGHT");
    scrub(
        later,
        look,
        line,
        theme::WEIGHT_START,
        theme::WEIGHT_STEP,
        theme::WEIGHT_RANGE,
    );
    let line = entry(later, look, held, "VISIBLE");
    checkbox(later, look, line, "", true);
    let line = entry(later, look, held, "QUALITY");
    listed(later, look, [line, over], &data::QUALITY, 2, false);
    let line = entry(later, look, held, "TINT");
    swatch(later, look, line, over, look.accent);
    let line = entry(later, look, held, "OPACITY");
    slider(later, look, line, theme::MASS_START);
}

fn lay_dock(later: &mut Later, look: &Theme, holder: Entity, shown: &mut Shown) {
    let page = page(later, look, holder, 8);
    relay(later, page, move |panel| panel.pad = 0.0);
    let strip = panel(
        later,
        page,
        Frame::column(look)
            .role(Dye::Header)
            .round(0.0)
            .pad(look.pad)
            .gap(look.gap * 0.5),
    );
    shown.dock_tree = Some(label(later, look, strip, "-", look.text));
    shown.dock_pointer = Some(small(later, look, strip, "-"));
    let deck = board(later, look, page);
    shown.deck = Some(deck);
    for (name, note) in data::PANES.iter() {
        let held = panel(
            later,
            deck,
            Frame::new(look)
                .along(Line::Start)
                .across(Line::Start)
                .role(Dye::Panel)
                .border(look.line)
                .rim(Dye::Edge)
                .round(0.0)
                .pad(look.pad),
        );
        label(later, look, held, name, look.heading);
        label(later, look, held, note, look.text);
        pane(later, deck, held, (name, note));
    }
}

fn page_of(later: &mut Later, look: &Theme, holder: Entity, index: usize) -> Entity {
    let held = page(later, look, holder, index);
    relay(later, held, move |panel| panel.pad = 0.0);
    scroll(
        later,
        look,
        held,
        Frame::column(look)
            .tall(Span::Fill(1.0))
            .bare()
            .pad(theme::PAGE_PAD)
            .gap(theme::BLOCK_GAP),
    )
}

fn block_of(later: &mut Later, look: &Theme, parent: Entity, heading: &str) -> Entity {
    let block = panel(
        later,
        parent,
        Frame::column(look)
            .role(Dye::Panel)
            .border(look.line)
            .rim(Dye::Edge)
            .lifted()
            .pad(theme::BLOCK_PAD)
            .gap(look.gap),
    );
    label(later, look, block, heading, look.heading);
    block
}

fn lay_data(later: &mut Later, look: &Theme, holder: Entity, shown: &mut Shown) {
    let data_page = page_of(later, look, holder, 4);
    let block = block_of(later, look, data_page, "A DATA GRID");
    relay(later, block, |panel| {
        panel.tall = Span::Fixed(theme::GRID_TALL)
    });
    let held = grid(later, look, block, &data::COLUMNS);
    shown.grid = Some(held);
    set(later, held, Streamed);
    let block = block_of(later, look, data_page, "A TREE");
    let mut stack: Vec<Entity> = vec![block];
    for (name, rung, leaf, glyph) in data::BRANCHES.iter() {
        stack.truncate(*rung as usize + 1);
        let Some(parent) = stack.last().copied() else {
            continue;
        };
        let sprig = Sprig {
            text: name,
            glyph: Some(*glyph),
            chosen: None,
        };
        match leaf {
            true => {
                twig(later, look, parent, sprig);
            }
            false => {
                let branched = branch(later, look, parent, sprig, data::MARKS, true);
                stack.push(branched.body);
            }
        }
    }

    let block = block_of(later, look, data_page, "A COLOR PICKER");
    picker(later, look, block, look.accent);
}

fn lay_out(later: &mut Later, look: &Theme, holder: Entity, over: Entity, shown: &mut Shown) {
    let out = page_of(later, look, holder, 5);
    let block = block_of(later, look, out, "A DATE PICKER");
    almanac(later, look, block, data::WHEN);

    let block = block_of(later, look, out, "A SPLIT");
    let holder_row = panel(
        later,
        block,
        Frame::new(look)
            .wide(Span::Fill(1.0))
            .tall(Span::Fixed(theme::SPLIT_TALL))
            .bare()
            .pad(0.0)
            .gap(0.0),
    );
    let (first, second, _) = split(later, look, holder_row, Lay::Row, theme::SPLIT_SHARE);
    label(later, look, first, "DRAG THE BAR", look.text);
    label(later, look, second, "EITHER WAY", look.text);
    let block = block_of(later, look, out, "A DRAG VALUE");
    scrub(
        later,
        look,
        block,
        theme::SCRUB_START,
        theme::SCRUB_STEP,
        theme::SCRUB_RANGE,
    );
    let block = block_of(later, look, out, "A CONTEXT MENU");
    let target = panel(
        later,
        block,
        Frame::new(look)
            .wide(Span::Fill(1.0))
            .tall(Span::Fixed(theme::MENU_TALL))
            .role(Dye::Input)
            .border(look.line)
            .rim(Dye::Edge)
            .pad(look.pad),
    );
    label(
        later,
        look,
        target,
        "RIGHT CLICK HERE, ESCAPE SHUTS IT BY CODE",
        look.text,
    );
    shown.menu_said = Some(small(later, look, target, "NOTHING CHOSEN"));
    touched(later, target);
    menu(later, look, over, target, &data::MENU_PICKS);
    shown.menu_target = Some(target);
}

fn lay_text(later: &mut Later, look: &Theme, holder: Entity, shown: &mut Shown) {
    let text_page = page_of(later, look, holder, 6);
    let block = block_of(later, look, text_page, "WRAPPED TEXT");
    wrapped(later, look, block, data::PARAGRAPH, look.text, theme::WRAP);
    let block = block_of(later, look, text_page, "A TEXT AREA");
    area(
        later,
        look,
        block,
        data::NOTE_TEXT,
        "write something",
        theme::WRAP,
        theme::AREA_TALL,
    );
    let block = block_of(later, look, text_page, "A CANVAS");
    shown.canvas = Some(canvas(
        later,
        look,
        block,
        Span::Fill(1.0),
        Span::Fixed(theme::CANVAS_TALL),
    ));

    let block = block_of(later, look, text_page, "COLOURED RUNS");
    runs(
        later,
        look,
        block,
        &[
            ("the ", Dye::Faint),
            ("quick ", Dye::Good),
            ("brown ", Dye::Warn),
            ("fox ", Dye::Bad),
            ("jumps", Dye::Accent),
        ],
    );
    let block = block_of(later, look, text_page, "TWO THOUSAND ROWS");
    let long = scroll(
        later,
        look,
        block,
        Frame::new(look)
            .wide(Span::Fill(1.0))
            .tall(Span::Fixed(theme::LONG_TALL))
            .along(Line::Start)
            .role(Dye::Input)
            .pad(look.pad * 0.5)
            .gap(0.0),
    );
    shown.long = Some(long);
}
