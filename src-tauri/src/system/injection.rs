//! Text injection strategy.
//!
//! The clipboard-based approach (PRD §5.5):
//!   1. Save current clipboard contents
//!   2. Put the new text on the clipboard
//!   3. Synthesize Cmd+V (CGEvent) into the frontmost app
//!   4. Restore the original clipboard after a short delay
//!
//! This is the VoiceInk / Wispr Flow approach and works in any focused app
//! that accepts text input — the canonical "inject anywhere" trick on macOS.

#[cfg(target_os = "macos")]
pub fn paste_into_focused_app(_text: &str) -> anyhow::Result<()> {
    // TODO: implement clipboard save → set → paste → restore.
    //   - Use `NSPasteboard generalPasteboard` via `objc` for clipboard.
    //   - Use `CGEventCreateKeyboardEvent` + `CGEventPost(kCGHIDEventTap, …)`
    //     for the synthetic Cmd+V.
    //   - Debounce the restore so the target app has time to consume the
    //     clipboard (~150ms is typical).
    anyhow::bail!("text injection not yet implemented")
}

#[cfg(not(target_os = "macos"))]
pub fn paste_into_focused_app(_text: &str) -> anyhow::Result<()> {
    anyhow::bail!("text injection only available on macOS")
}
