use crate::data::Way;
use ennui::prelude::Entity;

pub struct Walked {
    pub at: Option<Entity>,
    pub host: Option<Entity>,
    pub held: Option<Way>,
    pub waits: f32,
    pub steered: bool,
    pub stepped: bool,
    pub turned: Option<Way>,
}

impl Default for Walked {
    fn default() -> Self {
        Self {
            at: None,
            host: None,
            held: None,
            waits: 0.0,
            steered: true,
            stepped: false,
            turned: None,
        }
    }
}
