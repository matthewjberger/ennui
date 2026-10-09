use crate::data::Quad;
use ennui::prelude::Resources;
use std::any::TypeId;
use std::marker::PhantomData;

pub struct Quads<P> {
    pub list: Vec<Quad>,
    painter: PhantomData<fn() -> P>,
}

impl<P> Default for Quads<P> {
    fn default() -> Self {
        Self {
            list: Vec::new(),
            painter: PhantomData,
        }
    }
}

pub type Painted = fn(&Resources) -> &[Quad];

#[derive(Default)]
pub struct Painters {
    pub list: Vec<(TypeId, Painted)>,
}
