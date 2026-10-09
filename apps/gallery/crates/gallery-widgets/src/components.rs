use ennui::prelude::Entity;

#[derive(Clone, Copy, Default)]
pub(crate) struct Rank(pub usize);

#[derive(Clone, Copy, Default)]
pub struct Sorted {
    pub rank: usize,
    pub rising: bool,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct Head(pub Entity);

#[derive(Clone, Copy, Default)]
pub struct Stack(pub Entity);

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Streamed;

#[derive(Clone, Copy, Default)]
pub(crate) struct Divider {
    pub first: Entity,
    pub second: Entity,
    pub down: bool,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Ranged {
    pub low: f32,
    pub high: f32,
    pub grabbed: u8,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Grips {
    pub low: Entity,
    pub high: Entity,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Parts {
    pub before: Entity,
    pub middle: Entity,
    pub after: Entity,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Spun;

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Dated {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct Shifted(pub i32);
