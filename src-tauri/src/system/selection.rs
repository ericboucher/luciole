//! Read the current text selection from the focused app via Accessibility API.
//!
//! Requires the `AXIsProcessTrusted` entitlement granted by the user in
//! System Settings → Privacy & Security → Accessibility.

#[cfg(target_os = "macos")]
pub fn selected_text() -> anyhow::Result<String> {
    // TODO: AXUIElementCopyAttributeValue on the focused element for
    // `kAXSelectedTextAttribute`. Fall back to clipboard-copy trick
    // (Cmd+C → read clipboard → restore) when AX is not available.
    anyhow::bail!("selection read not yet implemented")
}

#[cfg(not(target_os = "macos"))]
pub fn selected_text() -> anyhow::Result<String> {
    anyhow::bail!("selection read only available on macOS")
}
