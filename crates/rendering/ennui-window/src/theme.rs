pub(crate) const WARM_FRAMES: usize = 60;
pub(crate) const HITCH_WINDOW: usize = 30;
pub(crate) const HITCH_RATIO: f32 = 2.0;
pub(crate) const FRAME_BUDGET: f32 = 1.0 / 144.0;
pub(crate) const WHEEL_PIXELS_EACH_LINE: f32 = 60.0;
pub(crate) const TOUCH_SLOP: f32 = 12.0;
pub(crate) const COAST_DECAY: f32 = 4.0;
pub(crate) const COAST_STOP: f32 = 20.0;
pub(crate) const SPEED_BLEND: f32 = 0.5;
pub(crate) const SMALLEST_DENSITY: f32 = 0.1;
pub(crate) const MILLISECONDS_EACH_SECOND: f32 = 1000.0;
pub(crate) const MILLIHERTZ: f32 = 1000.0;
#[cfg(target_arch = "wasm32")]
pub(crate) const CANVAS_ID: &str = "ennui";
#[cfg(target_arch = "wasm32")]
pub(crate) const SHELF_FILE: &str = "files.bin";
#[cfg(target_arch = "wasm32")]
#[cfg(target_arch = "wasm32")]
pub(crate) const DROPPED_FOLDER: &str = "dropped";
#[cfg(target_arch = "wasm32")]
pub(crate) const KEYBOARD_ATTRIBUTES: [(&str, &str); 6] = [
    ("autocapitalize", "off"),
    ("autocomplete", "off"),
    ("autocorrect", "off"),
    ("spellcheck", "false"),
    ("aria-hidden", "true"),
    (
        "style",
        "position:fixed;left:0;bottom:0;width:1px;height:1px;opacity:0;border:0;padding:0;font-size:16px",
    ),
];
