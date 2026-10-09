use crate::data::Screen;

pub const SCREENS: [(&str, &[Screen]); 4] = [
    (
        "backdrop",
        &[Screen::Title, Screen::Profile, Screen::Settings],
    ),
    ("title", &[Screen::Title]),
    ("profile", &[Screen::Profile]),
    ("settings", &[Screen::Settings]),
];

pub const SLIDE_TIME: f32 = 0.32;
pub const SLIDE_WIDTH: f32 = 90.0;
pub const BACK_SWING: f32 = 1.70158;
pub const STAMP_DELAY: f32 = 0.55;
pub const STAMP_TIME: f32 = 0.12;
pub const BAR_DELAY: f32 = 0.25;
pub const BAR_STAGGER: f32 = 0.12;
pub const BAR_TIME: f32 = 0.6;
pub const BAR_TRACK: f32 = 300.0;
pub const BAR_SHARES: [f32; 3] = [0.86, 0.64, 0.93];
pub const PULSE_SPEED: f32 = 3.2;
pub const LONGEST_STEP: f32 = 0.1;
