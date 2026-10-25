use serde::{Deserialize, Serialize};
use std::path::Path;
use tao::{
    dpi::{LogicalSize, PhysicalPosition},
    window::Window,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowGeometry {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub is_maximized: bool,
}

pub fn load_geometry(data_dir: &Path) -> Option<WindowGeometry> {
    let state_file = data_dir.join("window_state.json");
    if state_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&state_file) {
            if let Ok(geo) = serde_json::from_str::<WindowGeometry>(&content) {
                return Some(geo);
            }
        }
    }
    None
}

pub fn save_geometry(window: &Window, data_dir: &Path) {
    let pos = window.outer_position().unwrap_or(PhysicalPosition::new(100, 100));
    let size = window.inner_size();
    let is_maximized = window.is_maximized();

    let geo = WindowGeometry {
        x: pos.x,
        y: pos.y,
        width: size.width,
        height: size.height,
        is_maximized,
    };

    let state_file = data_dir.join("window_state.json");
    if let Ok(json) = serde_json::to_string_pretty(&geo) {
        let _ = std::fs::write(&state_file, json);
    }
}

pub fn apply_initial_placement(window: &Window, data_dir: &Path, _default_width: u32, _default_height: u32, persist: bool) {
    if persist {
        if let Some(geo) = load_geometry(data_dir) {
            // Verify if (x, y) is on an active monitor
            let is_on_active_monitor = window.available_monitors().any(|m| {
                let m_pos = m.position();
                let m_size = m.size();
                geo.x >= m_pos.x && geo.x < m_pos.x + m_size.width as i32 &&
                geo.y >= m_pos.y && geo.y < m_pos.y + m_size.height as i32
            });

            if is_on_active_monitor {
                window.set_outer_position(PhysicalPosition::new(geo.x, geo.y));
                window.set_inner_size(LogicalSize::new(geo.width, geo.height));
                if geo.is_maximized {
                    window.set_maximized(true);
                }
                return;
            }
        }
    }

    // Default: Center on current monitor
    if let Some(monitor) = window.current_monitor() {
        let screen_size = monitor.size();
        let window_size = window.outer_size();
        let x = (screen_size.width as i32 - window_size.width as i32) / 2;
        let y = (screen_size.height as i32 - window_size.height as i32) / 2;
        window.set_outer_position(PhysicalPosition::new(x, y));
    }
}
