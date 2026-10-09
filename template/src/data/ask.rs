use ennui_input::prelude::{Source, Worn};
use ennui_platform::prelude::KeyCode;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ask {
    Back,
}

pub const ASK_WORN: Worn<Ask> = &[(Ask::Back, &[Source::Key(KeyCode::Escape)])];
