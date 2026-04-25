//! Read the current text selection from the focused app via Accessibility API.
//!
//! Requires the `AXIsProcessTrusted` entitlement granted by the user in
//! System Settings → Privacy & Security → Accessibility.

#[cfg(target_os = "macos")]
#[allow(dead_code)]
pub fn selected_text() -> anyhow::Result<String> {
    // MVP: clipboard-copy trick (Cmd+C → read clipboard → restore).
    // AX path comes later; clipboard trick works across apps but needs AX permission.
    use std::{thread, time::Duration};

    if !crate::system::permissions::accessibility_granted() {
        anyhow::bail!("accessibility permission required to read selection");
    }

    let before = super::injection::clipboard_read_string();

    super::injection::post_cmd_keystroke(super::injection::KeyCode::C)?;
    thread::sleep(Duration::from_millis(140));

    let selected = super::injection::clipboard_read_string()
        .unwrap_or_default()
        .trim()
        .to_string();

    // restore
    match before {
        Some(s) => super::injection::clipboard_write_string(&s)?,
        None => super::injection::clipboard_clear()?,
    }

    if selected.is_empty() {
        anyhow::bail!("no selected text found (Cmd+C returned empty clipboard)");
    }
    Ok(selected)
}

#[cfg(not(target_os = "macos"))]
#[allow(dead_code)]
pub fn selected_text() -> anyhow::Result<String> {
    anyhow::bail!("selection read only available on macOS")
}
