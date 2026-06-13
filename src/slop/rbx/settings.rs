//! Boundary for engine settings singletons and a couple of process globals.
//!
//! EXTERNAL: `RBX::ClientAppSettings`, `RBX::GlobalBasicSettings`,
//! `RBX::GlobalAdvancedSettings`, `RBX::GameBasicSettings`,
//! `CRenderSettingsItem`, `::GetBaseURL` / `::SetBaseURL`.

#![allow(dead_code, unused_variables)]

use std::sync::RwLock;

static BASE_URL: RwLock<String> = RwLock::new(String::new());

pub fn base_url() -> String {
    BASE_URL.read().unwrap().clone()
}
pub fn set_base_url(url: &str) {
    *BASE_URL.write().unwrap() = url.to_owned();
}

/// `RBX::ClientAppSettings::singleton()`.
pub struct ClientAppSettings;
impl ClientAppSettings {
    pub fn singleton() -> Self {
        ClientAppSettings
    }
    pub fn http_use_curl_percentage_win_client(&self) -> i32 {
        0
    }
    pub fn google_analytics_init_fix(&self) -> bool {
        false
    }
    pub fn google_analytics_load_player(&self) -> i32 {
        0
    }
    pub fn allow_video_preroll(&self) -> bool {
        false
    }
    pub fn video_preroll_wait_time_seconds(&self) -> i32 {
        0
    }
}

/// `RBX::GlobalBasicSettings` / `RBX::GlobalAdvancedSettings`.
pub struct GlobalSettings;
impl GlobalSettings {
    pub fn basic() -> Self {
        GlobalSettings
    }
    pub fn advanced() -> Self {
        GlobalSettings
    }
    pub fn load_state(&self, path: &str) {}
    pub fn save_state(&self) {}
    pub fn remove_invalid_children(&self) {}
}

/// `RBX::GameBasicSettings::singleton()`.
pub struct GameBasicSettings;
impl GameBasicSettings {
    pub fn singleton() -> Self {
        GameBasicSettings
    }
    pub fn full_screen(&self) -> bool {
        false
    }
    pub fn set_full_screen(&self, value: bool) {}
    pub fn start_maximized(&self) -> bool {
        false
    }
    pub fn set_start_maximized(&self, value: bool) {}
    pub fn mouse_sensitivity(&self) -> f32 {
        1.0
    }
}
