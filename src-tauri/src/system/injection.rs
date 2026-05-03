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
#[allow(dead_code)]
pub fn paste_into_focused_app(text: &str) -> anyhow::Result<()> {
    use std::{thread, time::Duration};

    if !crate::system::permissions::accessibility_granted() {
        anyhow::bail!("accessibility permission required to inject text");
    }

    let before = clipboard_read_string();
    clipboard_write_string(text)?;

    post_cmd_keystroke(KeyCode::V)?;

    // Let frontmost app consume clipboard before restoring.
    thread::sleep(Duration::from_millis(180));
    match before {
        Some(s) => clipboard_write_string(&s)?,
        None => clipboard_clear()?,
    }

    Ok(())
}

#[cfg(not(target_os = "macos"))]
#[allow(dead_code)]
pub fn paste_into_focused_app(_text: &str) -> anyhow::Result<()> {
    anyhow::bail!("text injection only available on macOS")
}

#[cfg(target_os = "macos")]
#[derive(Clone, Copy)]
pub(crate) enum KeyCode {
    C,
    V,
}

#[cfg(target_os = "macos")]
pub(crate) fn post_cmd_keystroke(key: KeyCode) -> anyhow::Result<()> {
    use std::ffi::c_void;

    const K_CG_HID_EVENT_TAP: i32 = 0;
    const K_CG_EVENT_FLAG_MASK_COMMAND: u64 = 1 << 20;

    // US keyboard virtual keycodes.
    const KVK_ANSI_C: u16 = 0x08;
    const KVK_ANSI_V: u16 = 0x09;

    type CGEventRef = *mut c_void;
    type CGEventSourceRef = *mut c_void;

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn CGEventCreateKeyboardEvent(
            source: CGEventSourceRef,
            virtual_key: u16,
            key_down: bool,
        ) -> CGEventRef;
        fn CGEventSetFlags(event: CGEventRef, flags: u64);
        fn CGEventPost(tap: i32, event: CGEventRef);
        fn CFRelease(cf: *const c_void);
    }

    let vk = match key {
        KeyCode::C => KVK_ANSI_C,
        KeyCode::V => KVK_ANSI_V,
    };

    unsafe {
        let down = CGEventCreateKeyboardEvent(std::ptr::null_mut(), vk, true);
        if down.is_null() {
            anyhow::bail!("failed to create keydown event");
        }
        CGEventSetFlags(down, K_CG_EVENT_FLAG_MASK_COMMAND);
        CGEventPost(K_CG_HID_EVENT_TAP, down);
        CFRelease(down as *const c_void);

        let up = CGEventCreateKeyboardEvent(std::ptr::null_mut(), vk, false);
        if up.is_null() {
            anyhow::bail!("failed to create keyup event");
        }
        CGEventSetFlags(up, K_CG_EVENT_FLAG_MASK_COMMAND);
        CGEventPost(K_CG_HID_EVENT_TAP, up);
        CFRelease(up as *const c_void);
    }

    Ok(())
}

#[cfg(target_os = "macos")]
pub(crate) fn clipboard_read_string() -> Option<String> {
    #[allow(deprecated)]
    use cocoa::{
        appkit::{NSPasteboard, NSPasteboardTypeString},
        base::{id, nil},
    };
    use objc::runtime::Object;
    use objc::{msg_send, sel, sel_impl};

    unsafe {
        let pb: id = NSPasteboard::generalPasteboard(nil);
        #[allow(unexpected_cfgs)]
        let s: id = msg_send![pb, stringForType: NSPasteboardTypeString];
        if s == nil {
            return None;
        }
        let ns: *mut Object = s;
        #[allow(unexpected_cfgs)]
        let rust: *const std::os::raw::c_char = msg_send![ns, UTF8String];
        if rust.is_null() {
            return None;
        }
        Some(std::ffi::CStr::from_ptr(rust).to_string_lossy().to_string())
    }
}

#[cfg(target_os = "macos")]
pub(crate) fn clipboard_write_string(text: &str) -> anyhow::Result<()> {
    #[allow(deprecated)]
    use cocoa::{
        appkit::{NSPasteboard, NSPasteboardTypeString},
        base::{id, nil},
        foundation::NSString,
    };
    use objc::{msg_send, sel, sel_impl};

    unsafe {
        let pb: id = NSPasteboard::generalPasteboard(nil);
        #[allow(unexpected_cfgs)]
        let _: i64 = msg_send![pb, clearContents];
        let ns = NSString::alloc(nil).init_str(text);
        #[allow(unexpected_cfgs)]
        let ok: bool = msg_send![pb, setString: ns forType: NSPasteboardTypeString];
        if !ok {
            anyhow::bail!("failed to write to clipboard");
        }
    }
    Ok(())
}

#[cfg(target_os = "macos")]
pub(crate) fn clipboard_clear() -> anyhow::Result<()> {
    #[allow(deprecated)]
    use cocoa::{
        appkit::NSPasteboard,
        base::{id, nil},
    };
    use objc::{msg_send, sel, sel_impl};

    unsafe {
        let pb: id = NSPasteboard::generalPasteboard(nil);
        #[allow(unexpected_cfgs)]
        let _: i64 = msg_send![pb, clearContents];
    }
    Ok(())
}
