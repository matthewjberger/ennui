pub trait States: Clone + Copy + PartialEq + Send + Sync + 'static {}

impl<T: Clone + Copy + PartialEq + Send + Sync + 'static> States for T {}

pub struct Transition;
