use crate::resources::{FileOpened, Files};

pub fn ask_open(files: &Files, extensions: &[&str]) {
    let given = files.given.clone();
    let mut dialog = rfd::AsyncFileDialog::new();
    if !extensions.is_empty() {
        dialog = dialog.add_filter("files", extensions);
    }
    let picked = async move {
        if let Some(handle) = dialog.pick_file().await {
            let opened = FileOpened {
                name: handle.file_name(),
                bytes: handle.read().await,
            };
            given.send(opened).ok();
        }
    };
    #[cfg(not(target_arch = "wasm32"))]
    std::thread::spawn(move || pollster::block_on(picked));
    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(picked);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn ask_save(name: &str, bytes: Vec<u8>) {
    let dialog = rfd::AsyncFileDialog::new().set_file_name(name);
    std::thread::spawn(move || {
        pollster::block_on(async move {
            if let Some(handle) = dialog.save_file().await {
                handle.write(&bytes).await.ok();
            }
        })
    });
}

#[cfg(target_arch = "wasm32")]
pub fn ask_save(name: &str, bytes: Vec<u8>) {
    use wasm_bindgen::JsCast;
    let Some(document) = web_sys::window().and_then(|page| page.document()) else {
        return;
    };
    let parts = js_sys::Array::of1(&js_sys::Uint8Array::from(bytes.as_slice()));
    let Ok(blob) = web_sys::Blob::new_with_u8_array_sequence(&parts) else {
        return;
    };
    let Ok(link) = web_sys::Url::create_object_url_with_blob(&blob) else {
        return;
    };
    if let Ok(anchor) = document
        .create_element("a")
        .map(|element| element.unchecked_into::<web_sys::HtmlAnchorElement>())
    {
        anchor.set_href(&link);
        anchor.set_download(name);
        anchor.click();
    }
    web_sys::Url::revoke_object_url(&link).ok();
}

#[cfg(not(target_arch = "wasm32"))]
pub fn open_url(url: &str) -> bool {
    let mut command = match std::env::consts::OS {
        "windows" => {
            let mut held = std::process::Command::new("rundll32");
            held.arg("url.dll,FileProtocolHandler");
            held
        }
        "macos" => std::process::Command::new("open"),
        _ => std::process::Command::new("xdg-open"),
    };
    command.arg(url).spawn().is_ok()
}

#[cfg(target_arch = "wasm32")]
pub fn open_url(url: &str) -> bool {
    web_sys::window()
        .and_then(|page| page.open_with_url_and_target(url, "_blank").ok())
        .is_some()
}
