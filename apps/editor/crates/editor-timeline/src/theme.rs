use ennui_animation::prelude::Ease;
use nalgebra_glm::Vec4;

pub(crate) const NO_CLIP: &str =
    "Choose an entity with Play or Sequence to see its channels and keys";
pub(crate) const TRACK_TALL: f32 = 22.0;
pub(crate) const RULER_TALL: f32 = 16.0;
pub(crate) const KNOB: f32 = 10.0;
pub(crate) const HEAD_WIDE: f32 = 2.0;
pub(crate) const NAME_WIDE: f32 = 150.0;
pub(crate) const MARKS: usize = 5;
pub(crate) const MARK_ROOM: f32 = 28.0;
pub(crate) const KNOB_COLOR: Vec4 = Vec4::new(0.96, 0.70, 0.26, 1.0);
pub(crate) const HEAD_COLOR: Vec4 = Vec4::new(0.42, 0.71, 1.0, 0.9);
pub(crate) const SLIDER_NEAR: f32 = 1.0e-3;
pub(crate) const PLAY_TIP: &str = "Play or pause the clip in the editor window";
pub(crate) const STOP_TIP: &str = "Stop and put the authored values back";
pub(crate) const DELETE_TIP: &str = "Delete the chosen key";
pub(crate) const EASE_TIP: &str = "The curve from the chosen key to the next";
pub(crate) const TRACK_TIP: &str = "Click to add a key here, drag a key to move it";
pub(crate) const EASES: [(&str, Ease); 8] = [
    ("Linear", Ease::Linear),
    ("Step", Ease::Step),
    ("In", Ease::In),
    ("Out", Ease::Out),
    ("InOut", Ease::InOut),
    ("Smooth", Ease::Smooth),
    ("Back", Ease::Back),
    ("Bounce", Ease::Bounce),
];
pub(crate) const PLAY: &str = "Play";
pub(crate) const SEQUENCE: &str = "Sequence";
pub(crate) const CHANNEL: &str = "Channel";
pub(crate) const TIME_PLACES: usize = 2;
