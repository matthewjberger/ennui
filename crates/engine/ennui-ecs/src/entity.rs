#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct Entity {
    pub index: u32,
    pub generation: u32,
}
