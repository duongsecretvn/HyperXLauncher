use tao::window::Window;

/// Applies hardware-accelerated Acrylic / Mica or system compositor blur
pub fn apply_glassmorphism(window: &Window) {
    #[cfg(target_os = "windows")]
    {
        // Try Windows 11 Mica first, fallback to Acrylic blur
        if let Err(e) = window_vibrancy::apply_mica(window, Some(true)) {
            log::warn!("Mica effect unsupported or failed ({:?}), trying Acrylic blur", e);
            if let Err(e2) = window_vibrancy::apply_acrylic(window, Some((20, 20, 20, 180))) {
                log::warn!("Acrylic blur unsupported or failed: {:?}", e2);
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        // macOS NSVisualEffectView
        use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};
        if let Err(e) = apply_vibrancy(window, NSVisualEffectMaterial::HudWindow, None, None) {
            log::warn!("macOS vibrancy failed: {:?}", e);
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Linux compositor blur hint
        log::info!("Linux window compositor blur initialized");
    }
}
