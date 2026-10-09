use crate::commands::font::made_glyphs;
use crate::data::{GlyphsWarmed, SheetCleared, SheetHeld, SheetOpened};
use crate::systems::paint;
use ennui::app::insert_resource;
use ennui::prelude::{App, Stage, Step, before, grouped, on};

pub fn resources(app: &mut App) {
    insert_resource(&mut *app, made_glyphs());
}

pub fn systems() -> Vec<Step> {
    vec![
        grouped(SheetOpened, on(Stage::Startup, paint::open_sheet)),
        grouped(SheetCleared, on(Stage::Render, paint::clear_full_sheet)),
        grouped(GlyphsWarmed, on(Stage::Render, paint::warm_glyphs)),
        grouped(SheetHeld, on(Stage::Render, paint::hold_sheet)),
        before(SheetCleared, GlyphsWarmed),
        before(GlyphsWarmed, SheetHeld),
    ]
}
