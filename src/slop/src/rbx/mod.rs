//! # RBX engine boundary
//!
//! Every symbol in this module corresponds to something the original
//! `WindowsClient` C++ *called* but did **not** *define* — it lives in the
//! larger ROBLOX engine (`v8datamodel`, `gfxbase`, `network`, `util`, `rbx`,
//! `script`, `reflection`, …) or in a third-party library (Boost, G3D).
//!
//! Because none of that source is present in the `WindowsClient` directory,
//! a faithful, *compiling* port is impossible. This module therefore provides
//! idiomatic Rust **stand-ins** so the translated client code reads naturally
//! and type-checks. Bodies are stubs (`todo!()` / sensible defaults); link
//! them against the real engine to get a working binary.
//!
//! See `EXTERNAL_METHODS.md` for the full catalogue of these symbols.

#![allow(dead_code, unused_variables)]

use std::sync::Arc;

pub mod analytics;
pub mod http;
pub mod settings;

pub use http::{HttpFuture, HttpPostData};

/// `RBX::SharedLauncher::LaunchMode`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LaunchMode {
    Play,
    PlayProtocol,
    App,
}

/// `RBX::DataModelJob::*` access kind for `submitTask` / locks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobAccess {
    Read,
    Write,
    Render,
}

/// `RBX::Security::*` impersonation identity.
#[derive(Clone, Copy, Debug)]
pub enum SecurityIdentity {
    Com,
    RobloxGameScript,
}

/// Trait modelling `RBX::DataModel`. Only the members the client touches are
/// listed; the engine implements the rest.
pub trait DataModel: Send + Sync {
    fn is_closed(&self) -> bool;
    fn set_ui_message(&self, message: &str);
    fn clear_ui_message(&self);
    fn submit_task(&self, task: Box<dyn FnOnce() + Send>, access: JobAccess);
    fn find_gui_service(&self) -> Option<Arc<dyn GuiService>>;
    fn find_user_input_service(&self) -> Option<Arc<dyn UserInputService>>;
    fn get_screenshot_seo_info(&self) -> String;
    fn get_place_id(&self) -> i32;
}

/// Trait modelling `RBX::Game` / `RBX::SecurePlayerGame`.
pub trait Game: Send + Sync {
    fn get_data_model(&self) -> Option<Arc<dyn DataModel>>;
    fn shutdown(&self);
    fn configure_player(
        &self,
        identity: SecurityIdentity,
        config: String,
        launch_mode: LaunchMode,
        vr_device: Option<&str>,
    );
}

/// `RBX::GuiService` — the engine-side UI bridge the client connects to.
pub trait GuiService: Send + Sync {
    fn set_ui_message(&self, kind: u32, message: &str);
    /// `openUrlWindow` signal subscription.
    fn on_open_url_window(&self, handler: Box<dyn Fn(String) + Send + Sync>);
    /// `urlWindowClosed` signal subscription.
    fn on_url_window_closed(&self, handler: Box<dyn Fn() + Send + Sync>);
    fn url_window_closed(&self);
}

/// `RBX::UserInputService`.
pub trait UserInputService: Send + Sync {
    fn set_keyboard_enabled(&self, enabled: bool);
    fn set_mouse_enabled(&self, enabled: bool);
}

/// `RBX::SecurePlayerGame::SecurePlayerGame(...)` — construct the player game.
pub fn create_secure_player_game(base_url: &str) -> Arc<dyn Game> {
    todo!("RBX::SecurePlayerGame — defined in the engine, not in WindowsClient")
}

// ---------------------------------------------------------------------------
// Free functions / globals the client calls directly.
// ---------------------------------------------------------------------------

/// `::GetBaseURL()` / `::SetBaseURL()` (RobloxServicesTools).
pub fn base_url() -> String {
    settings::base_url()
}
pub fn set_base_url(url: &str) {
    settings::set_base_url(url);
}

/// `RBX::Game::globalInit` / `globalExit`.
pub fn game_global_init(studio: bool) {}
pub fn game_global_exit() {}

/// `RBX::ContentProvider::isUrl`.
pub fn content_is_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://") || url.starts_with("rbxasset")
}
/// `RBX::ContentProvider::setAssetFolder`.
pub fn content_set_asset_folder(folder: &str) {}

/// `RBX::Network::isTrustedContent`.
pub fn network_is_trusted_content(url: &str) -> bool {
    true
}

/// `RBX::format(...)` is just formatting — use Rust's `format!` at call sites.

/// `RBX::Time::nowFast().timestampSeconds()`.
pub fn now_fast_seconds() -> f64 {
    0.0
}
