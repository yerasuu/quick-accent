use std::process::Command;

pub struct ScreenInfo {
    pub width: f32,
    pub height: f32,
}

impl Default for ScreenInfo {
    fn default() -> Self {
        Self {
            width: 1920.0,
            height: 1080.0,
        }
    }
}

impl ScreenInfo {
    pub fn detect() -> Self {
        let result = Self::detect_linux().unwrap_or_default();
        
        // Debug output to verify detection
        eprintln!("Detected screen resolution: {}x{}", result.width, result.height);
        result
    }

    fn detect_linux() -> Result<Self, Box<dyn std::error::Error>> {
        // First try Wayland focused screen detection
        if let Ok(focused_screen) = Self::detect_wayland_focused_screen() {
            return Ok(focused_screen);
        }

        // Fallback to Wayland primary screen detection
        if let Ok(screen) = Self::detect_wayland_primary_screen() {
            return Ok(screen);
        }

        // Only use X11/XWayland as last resort when Wayland fails
        eprintln!("Wayland detection failed, falling back to X11/XWayland...");
        if let Ok(focused_screen) = Self::detect_x11_focused() {
            return Ok(focused_screen);
        }

        // Final fallback: try direct hardware detection
        if let Ok(resolution) = Self::detect_from_sysfs() {
            return Ok(resolution);
        }

        Err("Could not detect screen resolution on Linux".into())
    }

    fn detect_wayland_focused_screen() -> Result<ScreenInfo, Box<dyn std::error::Error>> {
        // Try Hyprland focused monitor first (most detailed focus detection)
        if let Ok(focused) = Self::detect_hyprland_focused() {
            eprintln!("Using Hyprland focused screen detection");
            return Ok(focused);
        }

        // Try Sway focused output
        if let Ok(focused) = Self::detect_sway_focused() {
            eprintln!("Using Sway focused screen detection");
            return Ok(focused);
        }

        Err("No Wayland focused screen detection available".into())
    }

    fn detect_wayland_primary_screen() -> Result<ScreenInfo, Box<dyn std::error::Error>> {
        // Try wlr-randr first (works with most wlroots-based compositors)
        if let Ok(output) = Command::new("wlr-randr").output() {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if let Some(resolution) = Self::parse_wlr_randr_output(&output_str) {
                    eprintln!("Using wlr-randr for primary screen detection");
                    return Ok(resolution);
                }
            }
        }

        // Try hyprctl for Hyprland (primary monitor)
        if let Ok(output) = Command::new("hyprctl").arg("monitors").arg("-j").output() {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if let Some(resolution) = Self::parse_hyprctl_output(&output_str) {
                    eprintln!("Using Hyprland for primary screen detection");
                    return Ok(resolution);
                }
            }
        }

        // Try swaymsg for Sway (primary output)
        if let Ok(output) = Command::new("swaymsg").arg("-t").arg("get_outputs").output() {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if let Some(resolution) = Self::parse_swaymsg_output(&output_str) {
                    eprintln!("Using Sway for primary screen detection");
                    return Ok(resolution);
                }
            }
        }

        Err("No Wayland primary screen detection available".into())
    }

    fn detect_hyprland_focused() -> Result<ScreenInfo, Box<dyn std::error::Error>> {
        // Get active monitor from Hyprland
        let output = Command::new("hyprctl")
            .arg("monitors")
            .arg("-j")
            .output()?;

        if !output.status.success() {
            return Err("hyprctl command failed".into());
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        
        // Look for focused monitor (the one with "focused": true)
        let mut in_focused_monitor = false;
        let mut width = None;
        let mut height = None;
        
        for line in output_str.lines() {
            let line = line.trim();
            
            if line.contains("\"focused\": true") {
                in_focused_monitor = true;
            } else if in_focused_monitor && line.starts_with("\"width\":") {
                if let Some(value_str) = line.split(':').nth(1) {
                    let value_str = value_str.trim().trim_end_matches(',');
                    width = value_str.parse().ok();
                }
            } else if in_focused_monitor && line.starts_with("\"height\":") {
                if let Some(value_str) = line.split(':').nth(1) {
                    let value_str = value_str.trim().trim_end_matches(',');
                    height = value_str.parse().ok();
                }
            } else if in_focused_monitor && line == "}" {
                if let (Some(w), Some(h)) = (width, height) {
                    return Ok(ScreenInfo { width: w, height: h });
                }
                break;
            }
        }
        
        Err("Could not find focused monitor in Hyprland".into())
    }

    fn detect_sway_focused() -> Result<ScreenInfo, Box<dyn std::error::Error>> {
        // Get focused workspace first
        let workspace_output = Command::new("swaymsg")
            .arg("-t")
            .arg("get_workspaces")
            .output()?;

        if !workspace_output.status.success() {
            return Err("swaymsg get_workspaces failed".into());
        }

        let workspace_str = String::from_utf8_lossy(&workspace_output.stdout);
        let mut focused_output_name = None;

        // Find the focused workspace and its output
        for line in workspace_str.lines() {
            let line = line.trim();
            if line.contains("\"focused\": true") {
                // Look for output field in the same object
                for search_line in workspace_str.lines() {
                    let search_line = search_line.trim();
                    if search_line.starts_with("\"output\":") {
                        if let Some(output_part) = search_line.split(':').nth(1) {
                            let output_name = output_part.trim().trim_matches('"').trim_end_matches(',');
                            focused_output_name = Some(output_name.to_string());
                            break;
                        }
                    }
                }
                break;
            }
        }

        if let Some(output_name) = focused_output_name {
            // Get resolution of that specific output
            let outputs_result = Command::new("swaymsg")
                .arg("-t")
                .arg("get_outputs")
                .output()?;

            if outputs_result.status.success() {
                let outputs_str = String::from_utf8_lossy(&outputs_result.stdout);
                if let Some(resolution) = Self::parse_sway_output_by_name(&outputs_str, &output_name) {
                    return Ok(resolution);
                }
            }
        }

        Err("Could not detect focused output in Sway".into())
    }

    fn parse_sway_output_by_name(output: &str, target_name: &str) -> Option<ScreenInfo> {
        let mut in_target_output = false;
        let mut in_current_mode = false;
        let mut width = None;
        let mut height = None;
        
        for line in output.lines() {
            let line = line.trim();
            
            if line.starts_with("\"name\":") && line.contains(target_name) {
                in_target_output = true;
            } else if in_target_output && line.contains("\"current\": true") {
                in_current_mode = true;
            } else if in_target_output && in_current_mode && line.starts_with("\"width\":") {
                if let Some(value_str) = line.split(':').nth(1) {
                    let value_str = value_str.trim().trim_end_matches(',');
                    width = value_str.parse().ok();
                }
            } else if in_target_output && in_current_mode && line.starts_with("\"height\":") {
                if let Some(value_str) = line.split(':').nth(1) {
                    let value_str = value_str.trim().trim_end_matches(',');
                    height = value_str.parse().ok();
                }
            } else if in_current_mode && line == "}" {
                if let (Some(w), Some(h)) = (width, height) {
                    return Some(ScreenInfo { width: w, height: h });
                }
                in_current_mode = false;
                width = None;
                height = None;
            } else if in_target_output && line == "]" {
                break;
            }
        }
        None
    }

    fn detect_x11_focused() -> Result<ScreenInfo, Box<dyn std::error::Error>> {
        eprintln!("Using X11/XWayland focused screen detection (mouse-based)");
        
        // Get mouse cursor position to determine which screen is focused
        let mouse_output = Command::new("xdotool")
            .arg("getmouselocation")
            .arg("--shell")
            .output();

        if let Ok(mouse_result) = mouse_output {
            if mouse_result.status.success() {
                let mouse_str = String::from_utf8_lossy(&mouse_result.stdout);
                let mut mouse_x = None;
                let mut mouse_y = None;

                for line in mouse_str.lines() {
                    if line.starts_with("X=") {
                        mouse_x = line[2..].parse().ok();
                    } else if line.starts_with("Y=") {
                        mouse_y = line[2..].parse().ok();
                    }
                }

                if let (Some(x), Some(y)) = (mouse_x, mouse_y) {
                    // Get screen info for the screen containing the mouse cursor
                    return Self::get_x11_screen_at_position(x, y);
                }
            }
        }

        // Fallback: get primary screen via xrandr
        eprintln!("Mouse detection failed, using X11 primary screen");
        let xrandr_output = Command::new("xrandr").arg("--current").output()?;
        if xrandr_output.status.success() {
            let output_str = String::from_utf8_lossy(&xrandr_output.stdout);
            if let Some(resolution) = Self::parse_xrandr_primary(&output_str) {
                return Ok(resolution);
            }
            // If no primary found, try any connected screen
            if let Some(resolution) = Self::parse_xrandr_any_connected(&output_str) {
                return Ok(resolution);
            }
        }

        Err("Could not detect X11 focused screen".into())
    }

    fn get_x11_screen_at_position(x: i32, y: i32) -> Result<ScreenInfo, Box<dyn std::error::Error>> {
        let output = Command::new("xrandr").arg("--current").output()?;
        
        if !output.status.success() {
            return Err("xrandr command failed".into());
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        eprintln!("X11 mouse position: {},{}  Looking for screen...", x, y);
        
        let mut closest_screen = None;
        let mut closest_distance = i32::MAX;
        
        // Parse xrandr output to find which screen contains the coordinates
        for line in output_str.lines() {
            if line.contains("connected") && !line.contains("disconnected") {
                eprintln!("Checking line: {}", line);
                let parts: Vec<&str> = line.split_whitespace().collect();
                for part in parts {
                    if part.contains("x") && part.contains("+") {
                        // Parse format like "1920x1080+1920+0" or "1080x1920+3840+0"
                        if let Some((res_part, pos_part)) = part.split_once('+') {
                            if let Some((width_str, height_str)) = res_part.split_once('x') {
                                if let Some((x_str, y_str)) = pos_part.split_once('+') {
                                    if let (Ok(width), Ok(height), Ok(screen_x), Ok(screen_y)) = (
                                        width_str.parse::<f32>(),
                                        height_str.parse::<f32>(),
                                        x_str.parse::<i32>(),
                                        y_str.parse::<i32>()
                                    ) {
                                        eprintln!("Found screen: {}x{} at {},{}", width, height, screen_x, screen_y);
                                        
                                        // Check if mouse is within this screen (exact match)
                                        if x >= screen_x && x < screen_x + width as i32 &&
                                           y >= screen_y && y < screen_y + height as i32 {
                                            eprintln!("Mouse is exactly on this screen!");
                                            return Ok(ScreenInfo { width, height });
                                        }
                                        
                                        // Calculate distance to this screen (for closest match)
                                        let distance_x = if x < screen_x {
                                            screen_x - x
                                        } else if x >= screen_x + width as i32 {
                                            x - (screen_x + width as i32 - 1)
                                        } else {
                                            0
                                        };
                                        
                                        let distance_y = if y < screen_y {
                                            screen_y - y
                                        } else if y >= screen_y + height as i32 {
                                            y - (screen_y + height as i32 - 1)
                                        } else {
                                            0
                                        };
                                        
                                        let total_distance = distance_x + distance_y;
                                        if total_distance < closest_distance {
                                            closest_distance = total_distance;
                                            closest_screen = Some(ScreenInfo { width, height });
                                            eprintln!("This is closest screen so far (distance: {})", total_distance);
                                        }
                                    }
                                }
                            }
                        }
                        break; // Found the resolution part, no need to check other parts of this line
                    }
                }
            }
        }

        // If we found a closest screen, use that
        if let Some(screen) = closest_screen {
            eprintln!("Using closest screen (distance: {})", closest_distance);
            return Ok(screen);
        }

        eprintln!("No screen found at mouse position, trying fallback...");
        Err("Could not find screen at position".into())
    }

    fn parse_xrandr_primary(output: &str) -> Option<ScreenInfo> {
        for line in output.lines() {
            if line.contains("primary") && line.contains("connected") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                for part in parts {
                    if part.contains("x") && part.contains("+") {
                        if let Some((res_part, _)) = part.split_once('+') {
                            if let Some((width_str, height_str)) = res_part.split_once('x') {
                                if let (Ok(width), Ok(height)) = (width_str.parse::<f32>(), height_str.parse::<f32>()) {
                                    return Some(ScreenInfo { width, height });
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn parse_xrandr_any_connected(output: &str) -> Option<ScreenInfo> {
        // Find any connected screen as fallback
        for line in output.lines() {
            if line.contains("connected") && !line.contains("disconnected") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                for part in parts {
                    if part.contains("x") && part.contains("+") {
                        if let Some((res_part, _)) = part.split_once('+') {
                            if let Some((width_str, height_str)) = res_part.split_once('x') {
                                if let (Ok(width), Ok(height)) = (width_str.parse::<f32>(), height_str.parse::<f32>()) {
                                    return Some(ScreenInfo { width, height });
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn parse_wlr_randr_output(output: &str) -> Option<ScreenInfo> {
        for line in output.lines() {
            if line.contains("current") {
                // Parse line like: "  1920x1080 px, 59.996002 Hz (current)"
                if let Some(resolution_part) = line.trim().split_whitespace().next() {
                    if let Some((width_str, height_str)) = resolution_part.split_once('x') {
                        if let (Ok(width), Ok(height)) = (width_str.parse::<f32>(), height_str.parse::<f32>()) {
                            return Some(ScreenInfo { width, height });
                        }
                    }
                }
            }
        }
        None
    }

    fn parse_hyprctl_output(output: &str) -> Option<ScreenInfo> {
        // Simple JSON parsing for Hyprland monitor info
        // Look for "width": and "height": fields
        let mut width = None;
        let mut height = None;
        
        for line in output.lines() {
            let line = line.trim();
            if line.starts_with("\"width\":") {
                if let Some(value_str) = line.split(':').nth(1) {
                    let value_str = value_str.trim().trim_end_matches(',');
                    width = value_str.parse().ok();
                }
            } else if line.starts_with("\"height\":") {
                if let Some(value_str) = line.split(':').nth(1) {
                    let value_str = value_str.trim().trim_end_matches(',');
                    height = value_str.parse().ok();
                }
            }
            
            // If we found both, return early
            if let (Some(w), Some(h)) = (width, height) {
                return Some(ScreenInfo { width: w, height: h });
            }
        }
        None
    }

    fn parse_swaymsg_output(output: &str) -> Option<ScreenInfo> {
        // Simple JSON parsing for Sway output info
        // Look for current mode with "width" and "height"
        let mut in_current_mode = false;
        let mut width = None;
        let mut height = None;
        
        for line in output.lines() {
            let line = line.trim();
            
            if line.contains("\"current\": true") {
                in_current_mode = true;
            } else if in_current_mode && line.starts_with("\"width\":") {
                if let Some(value_str) = line.split(':').nth(1) {
                    let value_str = value_str.trim().trim_end_matches(',');
                    width = value_str.parse().ok();
                }
            } else if in_current_mode && line.starts_with("\"height\":") {
                if let Some(value_str) = line.split(':').nth(1) {
                    let value_str = value_str.trim().trim_end_matches(',');
                    height = value_str.parse().ok();
                }
            }
            
            // Reset if we exit the current mode block
            if in_current_mode && line == "}" {
                if let (Some(w), Some(h)) = (width, height) {
                    return Some(ScreenInfo { width: w, height: h });
                }
                in_current_mode = false;
                width = None;
                height = None;
            }
        }
        None
    }

    fn detect_from_sysfs() -> Result<ScreenInfo, Box<dyn std::error::Error>> {
        use std::fs;
        
        // Try to read from /sys/class/drm/*/modes
        let drm_dir = "/sys/class/drm";
        if let Ok(entries) = fs::read_dir(drm_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("card") && name.contains("-") {
                        let modes_path = path.join("modes");
                        if let Ok(modes_content) = fs::read_to_string(&modes_path) {
                            if let Some(resolution) = Self::parse_drm_modes(&modes_content) {
                                return Ok(resolution);
                            }
                        }
                    }
                }
            }
        }
        
        Err("Could not detect from sysfs".into())
    }

    fn parse_drm_modes(content: &str) -> Option<ScreenInfo> {
        // Parse lines like "1920x1080"
        for line in content.lines() {
            let line = line.trim();
            if let Some((width_str, height_str)) = line.split_once('x') {
                if let (Ok(width), Ok(height)) = (width_str.parse::<f32>(), height_str.parse::<f32>()) {
                    // Return the first (usually highest) resolution
                    return Some(ScreenInfo { width, height });
                }
            }
        }
        None
    }

}
