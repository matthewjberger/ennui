#[derive(Clone, Default, PartialEq)]
pub struct Edit {
    pub text: String,
    pub at: usize,
    pub mark: usize,
}
