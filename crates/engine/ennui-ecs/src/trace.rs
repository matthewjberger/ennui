pub struct Traced {
    #[cfg(feature = "profiling")]
    held: Option<tracing::span::EnteredSpan>,
}

#[cfg(feature = "profiling")]
pub fn traced(name: &'static str) -> Traced {
    Traced {
        held: Some(tracing::info_span!("ennui", name).entered()),
    }
}

#[cfg(feature = "profiling")]
pub fn retrace(traced: &mut Traced, name: &'static str) {
    traced.held = None;
    traced.held = Some(tracing::info_span!("ennui", name).entered());
}

#[cfg(not(feature = "profiling"))]
pub fn traced(_name: &'static str) -> Traced {
    Traced {}
}

#[cfg(not(feature = "profiling"))]
pub fn retrace(_traced: &mut Traced, _name: &'static str) {}
