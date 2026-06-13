//! Translated from `Document.h/.cpp`.
//!
//! Owns the game state: constructs the `SecurePlayerGame`, wires the
//! `UserInputService`, runs the join script.
//!
//! ## Anti-cheat removed
//! The original `executeScript` injected `HATE_DEBUGGER` hack flags gated on
//! `VMProtectIsDebuggerPresent(...)`, and the file `#include`d
//! `VMProtect/VMProtectSDK.h`. Both are stripped here.

#![allow(dead_code)]

use std::sync::Arc;

use super::function_marshaller::FunctionMarshaller;
use super::rbx::{self, DataModel, Game, JobAccess, LaunchMode, SecurityIdentity};

/// `RBX::Document`.
pub struct Document {
    marshaller: *mut FunctionMarshaller,
    game: Option<Arc<dyn Game>>,
    /// `startedSignal` subscribers (bool = isTeleport).
    started_handlers: Vec<Box<dyn Fn(bool) + Send + Sync>>,
}

impl Document {
    pub fn new() -> Self {
        Self {
            marshaller: std::ptr::null_mut(),
            game: None,
            started_handlers: Vec::new(),
        }
    }

    pub fn game(&self) -> Option<Arc<dyn Game>> {
        self.game.clone()
    }

    pub fn marshaller(&self) -> *mut FunctionMarshaller {
        self.marshaller
    }

    pub fn on_started(&mut self, handler: Box<dyn Fn(bool) + Send + Sync>) {
        self.started_handlers.push(handler);
    }

    pub fn initialize(&mut self, _hwnd: windows::Win32::Foundation::HWND, use_chat: bool) {
        self.marshaller = FunctionMarshaller::get_window();
        let game = rbx::create_secure_player_game(&rbx::base_url());

        Self::configure_data_model_services(use_chat, game.get_data_model());

        // gameLoadedSignal.connect(&Document::gameIsLoaded) — engine signal.
        self.game = Some(game);
    }

    fn configure_data_model_services(_use_chat: bool, dm: Option<Arc<dyn DataModel>>) {
        let Some(dm) = dm else { return };
        if let Some(uis) = dm.find_user_input_service() {
            uis.set_keyboard_enabled(true);
            uis.set_mouse_enabled(true);
        }
    }

    /// `Document::Start`.
    pub fn start(
        &self,
        script_result: rbx::HttpFuture,
        launch_mode: LaunchMode,
        is_teleport: bool,
        vr_device: Option<&str>,
    ) {
        for h in &self.started_handlers {
            h(is_teleport);
        }
        rbx::analytics::report_event(rbx::analytics::EventType::Information, "Starting script");
        self.set_ui_message("");

        self.execute_script(script_result, launch_mode, vr_device);

        if !is_teleport {
            rbx::analytics::track_user_timing(
                "GAME",
                "CLIENT_START",
                rbx::now_fast_seconds() * 1000.0,
                "Join script executed",
            );
        }
    }

    pub fn set_ui_message(&self, message: &str) {
        if let Some(dm) = self.game.as_ref().and_then(|g| g.get_data_model()) {
            let message = message.to_owned();
            dm.submit_task(
                Box::new(move || {
                    // setUiMessageImpl
                }),
                JobAccess::Write,
            );
        }
    }

    /// `Document::executeScript` — anti-cheat / VMProtect stripped.
    fn execute_script(
        &self,
        script_result: rbx::HttpFuture,
        launch_mode: LaunchMode,
        vr_device: Option<&str>,
    ) {
        let Some(game) = self.game.clone() else {
            return;
        };
        let Some(dm) = game.get_data_model() else {
            return;
        };

        // [removed] dataModel->addHackFlag(HATE_DEBUGGER * VMProtectIsDebuggerPresent(...))

        // Security::Impersonator impersonate(Security::COM)
        let _identity = SecurityIdentity::Com;

        let data = match script_result.get() {
            Ok(d) => d,
            Err(e) => {
                rbx::analytics::report_event(
                    rbx::analytics::EventType::Error,
                    &format!("Exception in Document::executeScript: {e}"),
                );
                if let Some(gs) = dm.find_gui_service() {
                    gs.set_ui_message(0, "Unable to join game. Please try again later.");
                }
                return;
            }
        };

        // [removed] second VMProtectIsDebuggerPresent hack-flag block.

        // ProtectedString::fromTrustedSource + verifyScriptSignature (engine).
        if dm.is_closed() {
            return;
        }

        if let Some(idx) = data.find("\r\n") {
            let after = &data[idx + 2..];
            if after.starts_with('{') {
                game.configure_player(
                    SecurityIdentity::Com,
                    after.to_owned(),
                    launch_mode,
                    vr_device,
                );
            } else {
                // ScriptContext::executeInNewThread(Security::COM, verifiedSource, "Start Game")
                rbx_execute_in_new_thread(after);
            }
        }
    }

    pub fn prepare_shutdown(&self) {
        // setTimeout on ScriptContext when FLog::PlayerShutdownLuaTimeoutSeconds > 0
    }

    pub fn shutdown(&mut self) {
        if !self.marshaller.is_null() {
            FunctionMarshaller::release_window(self.marshaller);
            self.marshaller = std::ptr::null_mut();
        }
        if let Some(game) = self.game.take() {
            game.shutdown();
        }
    }

    /// `Document::GetSEOStr`.
    pub fn get_seo_str(&self) -> String {
        if let Some(dm) = self.game.as_ref().and_then(|g| g.get_data_model()) {
            let seo = dm.get_screenshot_seo_info();
            if !seo.is_empty() {
                return seo;
            }
        }
        // LoadStringA(IDS_DEFAULT_IMAGE_INFO)
        String::new()
    }
}

// EXTERNAL: RBX::ScriptContext::executeInNewThread — defined in script/.
fn rbx_execute_in_new_thread(_source: &str) {}
