use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ScreenConfig {
    /// Prefer focused screen over primary screen
    pub prefer_focused_screen: bool,
    /// Fallback to X11 when Wayland fails
    pub allow_x11_fallback: bool,
    /// Debug screen detection
    pub debug_screen_detection: bool,
}

impl Default for ScreenConfig {
    fn default() -> Self {
        Self {
            prefer_focused_screen: true,
            allow_x11_fallback: true,
            debug_screen_detection: true,
        }
    }
}
