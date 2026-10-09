use crate::data::{Bound, Cue, Drive, Ease, Easing, Key};
use ennui::prelude::Entity;

ennui::tuning! {
    #[derive(Clone, PartialEq, Debug, ennui::Reflect)]
    #[reflect(about = "An authored clip: its length, looping and cues; each child entity with a Channel drives one field")]
    pub struct Sequence {
        #[reflect(range(0.0, 600.0))]
        #[reflect(about = "Length of the clip in seconds")]
        length: f32 = 1.0,
        #[reflect(about = "Starts the clip again when it ends")]
        looping: bool = false,
        #[reflect(about = "Named moments that send a Marked event when the clip passes them")]
        cues: Vec<Cue> = Vec::new(),
    }
}

#[derive(Clone, Default, PartialEq, Debug, ennui::Reflect)]
#[reflect(about = "One driven field of the Sequence above it: the target field and its keys")]
pub struct Channel {
    #[reflect(
        about = "The field to drive, as entity/Component.field; the entity is a scene id under the playing entity or anywhere, and none means the playing entity"
    )]
    pub target: String,
    #[reflect(about = "Keys in time order, each with the value then and the curve to the next")]
    pub keys: Vec<Key>,
}

ennui::tuning! {
    #[derive(Clone, ennui::Reflect)]
    #[reflect(about = "Plays a Sequence on this entity, driving the fields its channels name over everything else that writes them")]
    pub struct Play {
        #[reflect(about = "Scene id of the Sequence entity, under this entity or anywhere in the scene")]
        clip: String = String::new(),
        #[reflect(range(-4.0, 4.0))]
        #[reflect(about = "Playback rate, 1 is normal, negative plays backward")]
        speed: f32 = 1.0,
        #[reflect(about = "Starts the clip again when it ends, as does the clip's own looping")]
        looping: bool = false,
        #[reflect(about = "The clip is playing; turns off by itself when a clip that does not loop ends")]
        playing: bool = true,
        #[reflect(range(0.0, 600.0))]
        #[reflect(about = "Time into the clip in seconds")]
        time: f32 = 0.0,
        #[reflect(skip)]
        bound: Vec<Bound> = Vec::new(),
        #[reflect(skip)]
        clip_entity: Option<Entity> = None,
        #[reflect(skip)]
        since: u64 = 0,
        #[reflect(skip)]
        last: f32 = 0.0,
    }
}

#[derive(Clone, Default, ennui::Reflect)]
#[reflect(
    about = "Eases fields of this entity from what they show to each new value another writer gives them"
)]
pub struct Tween {
    #[reflect(about = "The fields to ease, each with its time and curve")]
    pub fields: Vec<Eased>,
}

ennui::tuning! {
    #[derive(Clone, ennui::Reflect)]
    #[reflect(about = "One eased field of a Tween")]
    pub struct Eased {
        #[reflect(about = "The field on this entity, as Component.field")]
        field: String = String::new(),
        #[reflect(range(0.0, 60.0))]
        #[reflect(about = "Seconds the ease takes to reach a new value")]
        time: f32 = 0.25,
        #[reflect(about = "The curve of the ease")]
        ease: Ease = Ease::Out,
        #[reflect(skip)]
        easing: Easing = Easing::default(),
    }
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct Driven {
    pub fields: Vec<Drive>,
}
