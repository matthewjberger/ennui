#[cfg(not(target_arch = "wasm32"))]
pub fn read_clipboard() -> Option<String> {
    arboard::Clipboard::new().ok()?.get_text().ok()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn write_clipboard(text: &str) -> bool {
    arboard::Clipboard::new()
        .ok()
        .is_some_and(|mut held| held.set_text(text).is_ok())
}

#[cfg(target_arch = "wasm32")]
pub fn write_clipboard(text: &str) -> bool {
    web_sys::window().is_some_and(|window| {
        let _ = window.navigator().clipboard().write_text(text);
        true
    })
}
