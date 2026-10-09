pub mod commands;
mod components;
pub mod data;
pub mod plugin;
pub mod queries;
mod resources;
mod systems;
mod theme;

pub mod prelude {
    pub use crate::commands::bar::write_bar;
    pub use crate::commands::build::{
        area, bar, branch, button, buttons, checkbox, collapsing, confirm, desk, dim, entry, field,
        foot_bar, gauge, glide_card, head, hint, hint_rows, holder, knob, ledger, listed, menu,
        middle_card, modal, noted, offer, open_menu, page, palette, picker, pinned_card, plot,
        put_dropdown, put_field, put_plot, put_scrub, put_swatch, radio, raise, rest_of, rule,
        score_card, scrub, selectable, share_of, shut_menu, side_board, side_card, side_panel,
        slider, small, spread, sprout, stat, swatch, tabs, toast, toggle, tooltip, top_bar,
        track_of, tray_in, twig,
    };
    pub use crate::commands::progress::{clear_progress, set_progress, slide_progress};
    pub use crate::commands::switch::set_toggle;
    pub use crate::components::{
        Answer, Asked, Band, Button, Chose, Close, Drop, Dropdown, Entry, Field, Fold, Glide, Hint,
        Knob, Marked, Marks, Menu, Meter, Opened, Pick, Plot, Ran, Scrub, Sketch, Slide, Slider,
        Tab, Tabs, Toggle,
    };
    pub use crate::data::{
        Answered, Ask, Ease, Edit, Gauge, MIXED, Mark, Note, Offer, Roost, Seat, Series, Side,
        Sprig, Switching,
    };
    pub use crate::queries::keys::{chords, struck};
    pub use crate::queries::read::{
        asked, entered, head_of, menu_choice, opened, row_picked, tinted, wording,
    };
    pub use crate::queries::shape::{card_frame, edged, filling, plain};
    pub use crate::resources::{Asks, Progress, Tray};
    pub use crate::theme::{LEAST_FILL, PALETTE_SCOPES, SNUG, TREE_ROW};
}
