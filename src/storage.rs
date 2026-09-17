//! Tiny `localStorage` helpers for persisting UI preferences.
//!
//! Everything is best-effort: browsers can deny storage (private mode, disabled
//! cookies), so failures fall back to the caller's default and are ignored on write.

fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

pub fn load_bool(key: &str, default: bool) -> bool {
    storage()
        .and_then(|store| store.get_item(key).ok().flatten())
        .map(|value| value == "1")
        .unwrap_or(default)
}

pub fn save_bool(key: &str, value: bool) {
    if let Some(store) = storage() {
        let _ = store.set_item(key, if value { "1" } else { "0" });
    }
}
