use nalgebra_glm::Vec4;

pub(crate) const PROBLEM_SOURCE: &str = "ui";
pub(crate) const FRAME_INSET: f32 = 8.0;
pub(crate) const FRAME_TOP: f32 = 56.0;
pub(crate) const FRAME_COLOR: Vec4 = Vec4::new(0.95, 0.95, 0.95, 0.85);
pub(crate) const FRAME_FILL: Vec4 = Vec4::new(1.0, 1.0, 1.0, 0.04);
pub(crate) const FRAME_LINE: f32 = 2.0;
pub(crate) const SHEET_WIDE: f32 = 420.0;
pub(crate) const SHEET_DROP: f32 = 64.0;
pub(crate) const SHEET_LAYER: u32 = 1;
pub(crate) const SWATCH: f32 = 22.0;
pub(crate) const ASPECT_TIP: &str =
    "Frame the View pane to this aspect; Screen hosts lay out inside it";
pub(crate) const SCREEN_TIP: &str = "Show this screen read-only over the map at its real size";
pub(crate) const NO_SCREENS: &str = "No screens in scenes/ui";
pub(crate) const SHEET_TITLE: &str = "Theme sheet";
pub(crate) const SHEET_NOTE: &str = "Hover and press the controls for their live states";
pub(crate) const SLOTS: [(&str, ennui_ui::prelude::Dye); 13] = [
    ("ink", ennui_ui::prelude::Dye::Ink),
    ("faint", ennui_ui::prelude::Dye::Faint),
    ("accented", ennui_ui::prelude::Dye::Accented),
    ("ground", ennui_ui::prelude::Dye::Ground),
    ("panel", ennui_ui::prelude::Dye::Panel),
    ("header", ennui_ui::prelude::Dye::Header),
    ("edge", ennui_ui::prelude::Dye::Edge),
    ("accent", ennui_ui::prelude::Dye::Accent),
    ("good", ennui_ui::prelude::Dye::Good),
    ("warn", ennui_ui::prelude::Dye::Warn),
    ("bad", ennui_ui::prelude::Dye::Bad),
    ("input", ennui_ui::prelude::Dye::Input),
    ("chosen", ennui_ui::prelude::Dye::Chosen),
];
