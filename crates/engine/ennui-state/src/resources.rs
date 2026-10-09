use crate::data::States;

pub struct State<S> {
    pub current: S,
    pub next: Option<S>,
    pub from: Option<S>,
    pub turn: u64,
}

impl<S: States> State<S> {
    pub fn new(initial: S) -> Self {
        Self {
            current: initial,
            next: None,
            from: None,
            turn: 1,
        }
    }
}
