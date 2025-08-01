use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WindowConfig {
    /// Window width as fraction of screen width (0.0 to 1.0)
    pub width_fraction: f32,
    /// Window height in pixels
    pub height: f32,
    /// Window position: fraction of screen height from top (0.0 to 1.0)
    pub y_position_fraction: f32,
    /// Whether to center window horizontally
    pub center_horizontally: bool,
    /// Manual X offset if not centering (pixels)
    pub x_offset: f32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width_fraction: 0.75,
            height: 100.0,
            y_position_fraction: 0.25,
            center_horizontally: true,
            x_offset: 0.0,
        }
    }
}
