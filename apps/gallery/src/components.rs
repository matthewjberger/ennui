#[derive(Clone, Copy, Default)]
pub(crate) struct Named(pub &'static str);

#[derive(Clone, Copy, Default)]
pub(crate) struct Noted(pub usize);
