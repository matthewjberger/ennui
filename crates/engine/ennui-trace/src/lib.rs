pub const TRACE_FILE: &str = "trace.json";
pub const TRACE_ALSO: &str = "ENNUI_TRACE";

pub struct Traced {
    #[cfg(feature = "chrome")]
    _flushed: tracing_chrome::FlushGuard,
}

#[cfg(any(feature = "tracy", feature = "chrome"))]
pub fn start() -> Traced {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    let also: Vec<String> = std::env::var(TRACE_ALSO)
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|target| !target.is_empty())
        .map(String::from)
        .collect();
    let registry =
        tracing_subscriber::registry().with(tracing_subscriber::filter::filter_fn(move |held| {
            held.name() == "ennui"
                || also
                    .iter()
                    .any(|target| held.target().starts_with(target.as_str()))
        }));
    #[cfg(feature = "tracy")]
    let registry = registry.with(tracing_tracy::TracyLayer::default());
    #[cfg(feature = "chrome")]
    let (chrome, flushed) = tracing_chrome::ChromeLayerBuilder::new()
        .file(TRACE_FILE)
        .include_args(true)
        .build();
    #[cfg(feature = "chrome")]
    let registry = registry.with(chrome);
    registry.try_init().ok();
    Traced {
        #[cfg(feature = "chrome")]
        _flushed: flushed,
    }
}

#[cfg(not(any(feature = "tracy", feature = "chrome")))]
pub fn start() -> Traced {
    Traced {}
}
