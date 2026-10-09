use crate::components::{
    Button, Chose, Drop, Dropdown, Entered, Entry, Menu, Scrub, Slider, Tabs, Toggle,
};
use crate::data::{Answered, GlyphsDrawn, PaintedProgress, PaintedSketches, Switching};
use crate::resources::{Asks, Hinting, Progress, Tray};
use crate::systems::{
    answer, blink, drop, glide, hunt, menu, meter, palette, plot, progress, reveal, rule, scrub,
    show, sketch, skin, slide, sway, switch, sync, tint, toast, write,
};
use ennui::app::{component, insert_resource};
use ennui::prelude::{App, Stage, Step, before, grouped, on};
use ennui::storage::require;
use ennui_platform::data::Claiming;
use ennui_quads::prelude::add_painter;
use ennui_ui::data::{Arranged, Drawn, Floated, Focused, Pressed};
use ennui_ui::prelude::{Panel, Tone};

pub fn resources(app: &mut App) {
    add_painter::<PaintedSketches>(app);
    add_painter::<PaintedProgress>(app);
    insert_resource(&mut *app, Progress::default());
    insert_resource(&mut *app, Tray::default());
    insert_resource(&mut *app, Hinting::default());
    insert_resource(&mut *app, Asks::default());
    component::<Button>(app);
    component::<Toggle>(app);
    component::<Slider>(app);
    component::<Entry>(app);
    component::<Dropdown>(app);
    component::<Tabs>(app);
    require::<Scrub, Entered>(&mut app.storage);
    require::<Panel, Tone>(&mut app.storage);
    require::<Menu, Chose>(&mut app.storage);
    require::<Menu, Drop>(&mut app.storage);
}

pub fn systems() -> Vec<Step> {
    let answers = vec![
        on(Stage::Update, answer::read_answers),
        on(Stage::Update, answer::shut_modals),
        grouped(Switching, on(Stage::Update, switch::turn_boxes)),
        on(Stage::Update, slide::drag_slides),
        on(Stage::Update, slide::step_slides),
        grouped(GlyphsDrawn, on(Stage::Update, write::take_typing)),
        grouped(Switching, on(Stage::Update, switch::pick_radios)),
        grouped(Switching, on(Stage::Update, switch::mark_rows)),
        on(Stage::Update, drop::open_drops),
        on(Stage::Update, drop::shut_drops),
        on(Stage::Update, drop::take_picks),
        on(Stage::Update, show::turn_tabs),
        on(Stage::Update, show::fold_headers),
        on(Stage::Update, reveal::show_tips),
        on(Stage::Update, scrub::drag_scrubs),
        on(Stage::Update, menu::open_menus),
        on(Stage::Update, menu::shut_menus),
        on(Stage::Update, rule::drag_rulers),
        on(Stage::Update, hunt::filter_lists),
        on(Stage::Update, palette::steer_palettes),
        on(Stage::Update, sway::sway_toggles),
        grouped(GlyphsDrawn, on(Stage::Update, blink::blink_carets)),
        grouped(GlyphsDrawn, on(Stage::Update, blink::aim_carets)),
        grouped(GlyphsDrawn, on(Stage::Update, blink::draw_swaths)),
        grouped(GlyphsDrawn, on(Stage::Update, blink::follow_carets)),
        on(Stage::Update, tint::drag_wheels),
        on(Stage::Update, tint::turn_blends),
        on(Stage::Update, tint::mix_tones),
        on(Stage::Update, toast::age_toasts),
        on(Stage::Update, tint::place_dots),
        on(Stage::Update, sync::ask_buttons),
        on(Stage::Update, sync::sync_sliders),
        on(Stage::Update, sync::sync_entries),
        on(Stage::Update, sync::sync_dropdowns),
        on(Stage::Update, sync::sync_tabs),
    ];
    let rest = vec![
        on(Stage::Update, skin::skin_presses),
        on(Stage::Update, skin::skin_values),
        on(Stage::Update, skin::skin_choices),
        before(slide::drag_slides, sync::sync_sliders),
        before(slide::step_slides, sync::sync_sliders),
        before(write::take_typing, sync::sync_entries),
        before(drop::take_picks, sync::sync_dropdowns),
        before(show::turn_tabs, sync::sync_tabs),
        grouped(Claiming, on(Stage::Input, write::claim_keys)),
        before(Focused, write::claim_keys),
        on(Stage::Render, slide::fill_slides),
        on(Stage::Render, meter::show_meters),
        on(Stage::Render, glide::glide_cards),
        before(meter::show_meters, slide::fill_slides),
        before(glide::glide_cards, Floated),
        on(Stage::Render, plot::draw_plots),
        on(Stage::Render, sketch::paint_sketches),
        on(Stage::Render, progress::paint_progress),
        before(Arranged, plot::draw_plots),
        before(plot::draw_plots, sketch::paint_sketches),
        before(Drawn, sketch::paint_sketches),
        before(write::take_typing, scrub::drag_scrubs),
        before(write::take_typing, palette::steer_palettes),
        before(switch::turn_boxes, switch::pick_radios),
        before(slide::fill_slides, Arranged),
        before(Pressed, Answered),
    ];
    answers
        .into_iter()
        .map(|step| grouped(Answered, step))
        .chain(rest)
        .collect()
}
