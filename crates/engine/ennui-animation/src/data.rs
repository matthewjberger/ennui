use ennui::prelude::{Entity, Peek, Storage, View};
use ennui::reflect::prelude::Value;
use ennui_document::prelude::SceneId;
use ennui_scene::prelude::ChildOf;

pub(crate) struct Named<'held, 'row> {
    pub ids: &'held View<'held, (&'row SceneId,)>,
    pub parents: &'held Peek<'held, ChildOf>,
}

pub(crate) const EPSILON: f32 = 0.0001;
pub(crate) const LANES: usize = 4;
pub(crate) const SAME: f64 = 1.0e-6;
pub(crate) const BACK_OVERSHOOT: f32 = 1.70158;
pub(crate) const BOUNCE_PULL: f32 = 7.5625;
pub(crate) const BOUNCES: [(f32, f32, f32); 4] = [
    (1.0 / 2.75, 0.0, 0.0),
    (2.0 / 2.75, 1.5 / 2.75, 0.75),
    (2.5 / 2.75, 2.25 / 2.75, 0.9375),
    (2.0, 2.625 / 2.75, 0.984375),
];

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug, ennui::Reflect)]
pub enum Ease {
    #[default]
    Linear,
    Step,
    In,
    Out,
    InOut,
    Smooth,
    Back,
    Bounce,
}

#[derive(Clone, Default, PartialEq, Debug, ennui::Reflect)]
#[reflect(about = "One key of a channel: the time, the value there and the curve to the next key")]
pub struct Key {
    #[reflect(range(0.0, 600.0))]
    #[reflect(about = "Time of this key in seconds")]
    pub at: f32,
    #[reflect(about = "The field value at this key, written as the field is written in a scene")]
    pub value: Value,
    #[reflect(about = "The curve from this key to the next")]
    pub ease: Ease,
}

#[derive(Clone, Default, PartialEq, Debug, ennui::Reflect)]
#[reflect(about = "A named moment of a clip that sends a Marked event when the clip passes it")]
pub struct Cue {
    #[reflect(range(0.0, 600.0))]
    #[reflect(about = "Time of the cue in seconds")]
    pub at: f32,
    #[reflect(about = "The name the Marked event carries")]
    pub name: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Form {
    #[default]
    Other,
    Numbers(usize),
    Whole,
    Flag,
}

#[derive(Clone, Debug)]
pub struct Writer {
    pub entity: Entity,
    pub component: &'static str,
    pub path: String,
    pub depth: usize,
    pub read: fn(&Storage, Entity) -> Option<Value>,
    pub write: fn(&mut Storage, Entity, &Value) -> bool,
    pub skeleton: Value,
    pub form: Form,
}

#[derive(Clone, Debug)]
pub struct Bound {
    pub writer: Writer,
    pub times: Vec<f32>,
    pub values: Vec<f64>,
    pub eases: Vec<Ease>,
    pub held: Vec<Value>,
}

#[derive(Clone, Debug, Default)]
pub struct Easing {
    pub writer: Option<Writer>,
    pub from: [f64; LANES],
    pub target: [f64; LANES],
    pub shown: [f64; LANES],
    pub width: usize,
    pub elapsed: f32,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Drive {
    pub component: &'static str,
    pub path: String,
    pub value: Value,
}

#[derive(Clone, Copy)]
pub(crate) struct Strand {
    pub time: f32,
    pub last: f32,
    pub looping: bool,
}

#[derive(Clone)]
pub struct Marked {
    pub entity: Entity,
    pub name: String,
}

#[derive(Clone, Copy)]
pub struct Ended {
    pub entity: Entity,
}

pub struct Animated;
