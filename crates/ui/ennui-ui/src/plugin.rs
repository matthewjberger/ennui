use crate::components::{
    Click, Float, Hidden, Host, Hover, Lit, Order, Panel, Pin, Poke, Press, Sharp, Style, Sunk,
    Text, Touch, Turn, Warm,
};
use crate::data::{Arranged, Drawn, Dressed, Floated, PaintedPanels, Pressed, Written};
use crate::resources::{
    Fonts, Hosts, Interface, Laid, Letterbox, Pictures, Said, Steering, Theme, Wearing,
};
use crate::systems::{dress, ease, layout, load, moor, paint, press, ride, texts, wear};
use ennui::app::{component, insert_resource};
use ennui::prelude::{App, Stage, Step, before, grouped, on, settling};
use ennui::reflect::prelude::resource;
use ennui::storage::require;
use ennui_document::prelude::{Level, Outsiders};
use ennui_platform::data::Claiming;
use ennui_quads::prelude::add_painter;
use ennui_text::data::{SheetCleared, SheetHeld};
use ennui_watch::prelude::{Changes, Watch};

pub fn resources(app: &mut App) {
    add_painter::<PaintedPanels>(app);
    ennui::resources::hold::<ennui_document::prelude::AssetLibrary>(&mut app.resources);
    insert_resource(&mut *app, Theme::default());
    insert_resource(&mut *app, Interface::default());
    insert_resource(&mut *app, Letterbox::default());
    insert_resource(&mut *app, Laid::default());
    insert_resource(&mut *app, Hosts::default());
    insert_resource(&mut *app, Fonts::default());
    insert_resource(&mut *app, Pictures::default());
    insert_resource(&mut *app, Said::default());
    insert_resource(&mut *app, Steering::default());
    insert_resource(&mut *app, Wearing::default());
    ennui::resources::hold::<Level>(&mut app.resources);
    ennui::resources::hold::<Outsiders>(&mut app.resources);
    ennui::resources::hold::<Watch>(&mut app.resources);
    ennui::resources::hold::<Changes>(&mut app.resources);
    resource::<Interface>(&mut app.resources);
    component::<Host>(app);
    component::<Panel>(app);
    component::<Pin>(app);
    component::<Turn>(app);
    component::<Style>(app);
    component::<Text>(app);
    component::<Theme>(app);
    component::<Hidden>(app);
    component::<Order>(app);
    component::<Sharp>(app);
    require::<Panel, Hidden>(&mut app.storage);
    require::<Panel, Lit>(&mut app.storage);
    require::<Text, Panel>(&mut app.storage);
    require::<Text, Style>(&mut app.storage);
    require::<Pin, Float>(&mut app.storage);
    require::<Touch, Poke>(&mut app.storage);
    require::<Touch, Hover>(&mut app.storage);
    require::<Touch, Press>(&mut app.storage);
    require::<Touch, Click>(&mut app.storage);
    require::<Touch, Warm>(&mut app.storage);
    require::<Touch, Sunk>(&mut app.storage);
}

pub fn systems() -> Vec<Step> {
    vec![
        grouped(
            Claiming,
            grouped(Pressed, on(Stage::Input, press::read_presses)),
        ),
        on(Stage::Update, ride::drag_thumbs),
        on(Stage::Update, ease::ease_states),
        on(Stage::Update, wear::wear_theme),
        settling::<Interface>(),
        on(Stage::Render, load::load_fonts),
        on(Stage::Render, load::load_pictures),
        grouped(Written, on(Stage::Render, texts::write_texts)),
        grouped(Dressed, on(Stage::Render, dress::dress_panels)),
        on(Stage::Render, moor::drop_strays),
        on(Stage::Render, moor::follow_anchors),
        before(moor::drop_strays, moor::follow_anchors),
        grouped(Floated, on(Stage::Render, moor::pin_the_floats)),
        on(Stage::Render, ride::ride_thumbs),
        on(Stage::Render, layout::scroll_panels),
        grouped(Arranged, on(Stage::Render, layout::arrange)),
        grouped(Drawn, on(Stage::Render, paint::paint_panels)),
        before(load::load_fonts, Written),
        before(Written, Dressed),
        before(Dressed, Arranged),
        before(moor::follow_anchors, Arranged),
        before(Floated, Arranged),
        before(ride::ride_thumbs, Arranged),
        before(layout::scroll_panels, Arranged),
        before(layout::scroll_panels, ride::ride_thumbs),
        before(Arranged, Drawn),
        before(SheetCleared, Written),
        before(Drawn, SheetHeld),
        before(load::load_pictures, Drawn),
    ]
}
