use crate::data::Screen;

pub const SCREENS: [(&str, &[Screen]); 2] =
    [("home", &[Screen::Home]), ("settings", &[Screen::Settings])];
