//! System-wide selection monitor + overlay anchoring (macOS).
//!
//! MVP: poll Accessibility selection + compute best-effort anchor rect.

use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AnchorRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OverlayState {
    pub visible: bool,
    pub anchor: Option<AnchorRect>,
}

#[cfg(target_os = "macos")]
pub fn start(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut last: Option<OverlayState> = None;
        // Fallback anchor when we can't compute selection bounds.
        // We freeze it for given selection signature so overlay stays clickable.
        let mut frozen_fallback_anchor: Option<AnchorRect> = None;
        let mut last_selection_sig: Option<String> = None;
        loop {
            let next = match poll_state(&mut frozen_fallback_anchor, &mut last_selection_sig) {
                Ok(s) => s,
                Err(_) => OverlayState {
                    visible: false,
                    anchor: None,
                },
            };

            if last.as_ref() != Some(&next) {
                let _ = app.emit("overlay:state", &next);
                last = Some(next);
            }

            tokio::time::sleep(std::time::Duration::from_millis(180)).await;
        }
    });
}

#[cfg(not(target_os = "macos"))]
pub fn start(_app: AppHandle) {}

#[cfg(target_os = "macos")]
fn poll_state(
    frozen_fallback_anchor: &mut Option<AnchorRect>,
    last_selection_sig: &mut Option<String>,
) -> anyhow::Result<OverlayState> {
    if !super::permissions::accessibility_granted() {
        *frozen_fallback_anchor = None;
        *last_selection_sig = None;
        return Ok(OverlayState {
            visible: false,
            anchor: None,
        });
    }

    // Non-invasive: check selection via AX. Some apps omit AXSelectedTextRange.
    if !ax::has_selection() && !ax::has_selected_text() {
        *frozen_fallback_anchor = None;
        *last_selection_sig = None;
        return Ok(OverlayState {
            visible: false,
            anchor: None,
        });
    }

    // Prefer bounds for selection; if unavailable, fall back to *frozen* mouse anchor
    // (captured once per selection) to avoid following cursor.
    let anchor = if let Some(bounds) = ax::selection_bounds() {
        *frozen_fallback_anchor = None;
        *last_selection_sig = None;
        Some(bounds)
    } else {
        let sig = ax::selection_signature();
        if sig.is_none() {
            // no stable signature, keep existing frozen anchor if any
            frozen_fallback_anchor.or_else(ax::mouse_anchor)
        } else {
            let sig = sig.unwrap();
            if last_selection_sig.as_deref() != Some(&sig) {
                *last_selection_sig = Some(sig);
                *frozen_fallback_anchor = ax::mouse_anchor();
            }
            *frozen_fallback_anchor
        }
    };
    Ok(OverlayState {
        visible: anchor.is_some(),
        anchor,
    })
}

#[cfg(target_os = "macos")]
mod ax {
    use super::AnchorRect;
    use core_foundation::{base::{CFRelease, TCFType}, string::CFString};
    use std::ffi::c_void;

    type AXUIElementRef = *const c_void;
    type AXValueRef = *const c_void;
    type CFTypeRef = *const c_void;
    type CFStringRef = *const c_void;

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct CGPoint {
        x: f64,
        y: f64,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct CGSize {
        width: f64,
        height: f64,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct CGRect {
        origin: CGPoint,
        size: CGSize,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct CFRange {
        location: i64,
        length: i64,
    }

    #[allow(non_camel_case_types)]
    type AXValueType = i32;
    const K_AX_VALUE_CFRANGE_TYPE: AXValueType = 4;
    const K_AX_VALUE_CGRECT_TYPE: AXValueType = 2;

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXUIElementCreateSystemWide() -> AXUIElementRef;
        fn AXUIElementCopyAttributeValue(
            element: AXUIElementRef,
            attribute: CFStringRef,
            value: *mut CFTypeRef,
        ) -> i32;
        fn AXUIElementCopyParameterizedAttributeValue(
            element: AXUIElementRef,
            parameterized_attribute: CFStringRef,
            parameter: CFTypeRef,
            result: *mut CFTypeRef,
        ) -> i32;

        fn AXValueGetValue(value: AXValueRef, the_type: AXValueType, value_ptr: *mut c_void) -> bool;
    }

    fn cfstr(s: &str) -> CFString {
        CFString::new(s)
    }

    unsafe fn copy_attr(element: AXUIElementRef, attr: &CFString) -> Option<CFTypeRef> {
        let mut out: CFTypeRef = std::ptr::null();
        let status = AXUIElementCopyAttributeValue(element, attr.as_concrete_TypeRef() as _, &mut out);
        if status == 0 && !out.is_null() {
            Some(out)
        } else {
            None
        }
    }

    unsafe fn copy_param_attr(
        element: AXUIElementRef,
        attr: &CFString,
        param: CFTypeRef,
    ) -> Option<CFTypeRef> {
        let mut out: CFTypeRef = std::ptr::null();
        let status = AXUIElementCopyParameterizedAttributeValue(
            element,
            attr.as_concrete_TypeRef() as _,
            param,
            &mut out,
        );
        if status == 0 && !out.is_null() {
            Some(out)
        } else {
            None
        }
    }

    pub fn has_selection() -> bool {
        unsafe {
            let system = AXUIElementCreateSystemWide();
            if system.is_null() {
                return false;
            }

            let focused_attr = cfstr("AXFocusedUIElement");
            let focused = match copy_attr(system, &focused_attr) {
                Some(v) => v as AXUIElementRef,
                None => return false,
            };

            let range_attr = cfstr("AXSelectedTextRange");
            let range_val = match copy_attr(focused, &range_attr) {
                Some(v) => v as AXValueRef,
                None => {
                    CFRelease(focused as _);
                    return false;
                }
            };

            let mut range: CFRange = std::mem::zeroed();
            let ok = AXValueGetValue(
                range_val,
                K_AX_VALUE_CFRANGE_TYPE,
                &mut range as *mut _ as *mut c_void,
            );

            CFRelease(focused as _);
            CFRelease(range_val as _);

            ok && range.length > 0
        }
    }

    pub fn has_selected_text() -> bool {
        unsafe {
            let system = AXUIElementCreateSystemWide();
            if system.is_null() {
                return false;
            }

            let focused_attr = cfstr("AXFocusedUIElement");
            let focused = match copy_attr(system, &focused_attr) {
                Some(v) => v as AXUIElementRef,
                None => return false,
            };

            let sel_attr = cfstr("AXSelectedText");
            let value = match copy_attr(focused, &sel_attr) {
                Some(v) => v,
                None => {
                    CFRelease(focused as _);
                    return false;
                }
            };

            let cf = core_foundation::base::CFType::wrap_under_create_rule(value as _);
            let s = CFString::wrap_under_get_rule(cf.as_CFTypeRef() as _);
            let out = s.to_string();

            CFRelease(focused as _);
            !out.trim().is_empty()
        }
    }

    pub fn selection_bounds() -> Option<AnchorRect> {
        unsafe {
            let system = AXUIElementCreateSystemWide();
            if system.is_null() {
                return None;
            }

            let focused_attr = cfstr("AXFocusedUIElement");
            let focused = copy_attr(system, &focused_attr)? as AXUIElementRef;

            let range_attr = cfstr("AXSelectedTextRange");
            let range_val = copy_attr(focused, &range_attr)? as AXValueRef;

            // Use AXBoundsForRange parameterized attribute
            let bounds_attr = cfstr("AXBoundsForRange");
            let rect_val = copy_param_attr(focused, &bounds_attr, range_val as _)? as AXValueRef;

            let mut rect: CGRect = std::mem::zeroed();
            let ok = AXValueGetValue(rect_val, K_AX_VALUE_CGRECT_TYPE, &mut rect as *mut _ as _);

            // Cleanup (create rule for focused, range_val, rect_val).
            CFRelease(focused as _);
            CFRelease(range_val as _);
            CFRelease(rect_val as _);

            if !ok {
                return None;
            }

            Some(AnchorRect {
                x: rect.origin.x,
                y: rect.origin.y,
                width: rect.size.width.max(1.0),
                height: rect.size.height.max(1.0),
            })
        }
    }

    pub fn selection_signature() -> Option<String> {
        // Best-effort: use selected range (location/length). If not available, fallback to selected text length.
        unsafe {
            let system = AXUIElementCreateSystemWide();
            if system.is_null() {
                return None;
            }

            let focused_attr = cfstr("AXFocusedUIElement");
            let focused = match copy_attr(system, &focused_attr) {
                Some(v) => v as AXUIElementRef,
                None => return None,
            };

            let range_attr = cfstr("AXSelectedTextRange");
            if let Some(v) = copy_attr(focused, &range_attr) {
                let range_val = v as AXValueRef;
                let mut range: CFRange = std::mem::zeroed();
                let ok = AXValueGetValue(
                    range_val,
                    K_AX_VALUE_CFRANGE_TYPE,
                    &mut range as *mut _ as *mut c_void,
                );
                CFRelease(focused as _);
                CFRelease(range_val as _);
                if ok && range.length > 0 {
                    return Some(format!("range:{}:{}", range.location, range.length));
                }
            }

            let sel_attr = cfstr("AXSelectedText");
            let value = match copy_attr(focused, &sel_attr) {
                Some(v) => v,
                None => {
                    CFRelease(focused as _);
                    return None;
                }
            };

            let cf = core_foundation::base::CFType::wrap_under_create_rule(value as _);
            let s = CFString::wrap_under_get_rule(cf.as_CFTypeRef() as _);
            let out = s.to_string();
            CFRelease(focused as _);
            Some(format!("textlen:{}", out.len()))
        }
    }

    pub fn mouse_anchor() -> Option<AnchorRect> {
        use core_graphics::{
            event::CGEvent,
            event_source::{CGEventSource, CGEventSourceStateID},
        };
        let src = CGEventSource::new(CGEventSourceStateID::CombinedSessionState).ok()?;
        let e = CGEvent::new(src).ok()?;
        let p = e.location();
        Some(AnchorRect {
            x: p.x,
            y: p.y,
            width: 1.0,
            height: 1.0,
        })
    }
}

