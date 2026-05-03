//! macOS privacy permission probes.
//!
//! These are best-effort — Apple does not expose a stable public API for
//! checking every permission state, so we combine TCC-friendly probes.

#[cfg(target_os = "macos")]
#[allow(unexpected_cfgs)]
pub fn microphone_granted() -> bool {
    use objc::{msg_send, runtime::Class, sel, sel_impl};
    unsafe {
        let cls = match Class::get("AVCaptureDevice") {
            Some(c) => c,
            None => return false,
        };
        // AVAuthorizationStatus: notDetermined=0, restricted=1, denied=2, authorized=3
        let media_type = ns_string("soun");
        let status: i64 = msg_send![cls, authorizationStatusForMediaType: media_type];
        status == 3
    }
}

#[cfg(not(target_os = "macos"))]
pub fn microphone_granted() -> bool {
    true
}

#[cfg(target_os = "macos")]
#[allow(unexpected_cfgs)]
pub fn prompt_microphone() -> anyhow::Result<()> {
    use block::ConcreteBlock;
    use objc::{msg_send, runtime::Class, sel, sel_impl};

    unsafe {
        let cls = Class::get("AVCaptureDevice").ok_or_else(|| anyhow::anyhow!("AVCaptureDevice not found"))?;
        let media_type = ns_string("soun");

        // completionHandler: (BOOL granted) -> void
        let handler = ConcreteBlock::new(|_granted: bool| {}).copy();
        let _: () = msg_send![cls, requestAccessForMediaType: media_type completionHandler: &*handler];
        Ok(())
    }
}

#[cfg(not(target_os = "macos"))]
pub fn prompt_microphone() -> anyhow::Result<()> {
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn accessibility_granted() -> bool {
    use core_foundation::base::TCFType;
    use core_foundation::dictionary::CFDictionary;
    use core_foundation::string::CFString;

    extern "C" {
        fn AXIsProcessTrustedWithOptions(
            options: core_foundation::dictionary::CFDictionaryRef,
        ) -> bool;
    }

    unsafe {
        let key = CFString::new("AXTrustedCheckOptionPrompt");
        let value = core_foundation::boolean::CFBoolean::false_value();
        let dict = CFDictionary::from_CFType_pairs(&[(key, value)]);
        AXIsProcessTrustedWithOptions(dict.as_concrete_TypeRef())
    }
}

#[cfg(not(target_os = "macos"))]
pub fn accessibility_granted() -> bool {
    true
}

#[cfg(target_os = "macos")]
pub fn prompt_accessibility() -> anyhow::Result<()> {
    use core_foundation::base::TCFType;
    use core_foundation::dictionary::CFDictionary;
    use core_foundation::string::CFString;

    extern "C" {
        fn AXIsProcessTrustedWithOptions(
            options: core_foundation::dictionary::CFDictionaryRef,
        ) -> bool;
    }

    unsafe {
        let key = CFString::new("AXTrustedCheckOptionPrompt");
        let value = core_foundation::boolean::CFBoolean::true_value();
        let dict = CFDictionary::from_CFType_pairs(&[(key, value)]);
        AXIsProcessTrustedWithOptions(dict.as_concrete_TypeRef());
    }
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn prompt_accessibility() -> anyhow::Result<()> {
    Ok(())
}

#[cfg(target_os = "macos")]
#[allow(unexpected_cfgs)]
unsafe fn ns_string(s: &str) -> *mut objc::runtime::Object {
    use objc::{msg_send, runtime::Class, sel, sel_impl};
    let cls = Class::get("NSString").unwrap();
    let bytes = s.as_ptr();
    let len = s.len();
    msg_send![cls,
        stringWithBytes: bytes
        length: len
        encoding: 4usize /* NSUTF8StringEncoding */]
}
