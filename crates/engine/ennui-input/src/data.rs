use ennui_platform::prelude::{Input, KeyCode, MouseButton};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub struct ActionsRead;

pub(crate) struct Heard<'a> {
    pub input: &'a Input,
    pub pointed: bool,
    pub typing: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Source {
    Key(KeyCode),
    Pointer(MouseButton),
    Pair {
        less: KeyCode,
        more: KeyCode,
    },
    Cross {
        less: KeyCode,
        more: KeyCode,
        down: KeyCode,
        up: KeyCode,
    },
    Wheel,
}

pub trait Action: Clone + Copy + Debug + PartialEq + Send + Sync + 'static {}
impl<T: Clone + Copy + Debug + PartialEq + Send + Sync + 'static> Action for T {}

pub type Worn<A> = &'static [(A, &'static [Source])];
