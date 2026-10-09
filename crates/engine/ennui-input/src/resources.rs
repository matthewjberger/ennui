use crate::data::{Action, Source, Worn};
use nalgebra_glm::Vec2;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default)]
pub struct Book {
    pub profile: String,
    pub profiles: BTreeMap<String, BTreeMap<String, Vec<Source>>>,
}

pub struct Bindings<A: Action> {
    pub worn: Vec<(A, Vec<Source>)>,
    pub stock: Worn<A>,
    pub book: Book,
    pub file: Option<PathBuf>,
    pub changed: Option<f32>,
}

pub struct Actions<A: Action> {
    pub held: Vec<A>,
    pub pressed: Vec<A>,
    pub released: Vec<A>,
    pub axes: Vec<(A, Vec2)>,
    pub steps: Vec<(A, Vec2)>,
}

impl<A: Action> Default for Actions<A> {
    fn default() -> Self {
        Self {
            held: Vec::new(),
            pressed: Vec::new(),
            released: Vec::new(),
            axes: Vec::new(),
            steps: Vec::new(),
        }
    }
}
