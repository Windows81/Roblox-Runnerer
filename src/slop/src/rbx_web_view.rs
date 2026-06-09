//! Translated from `RbxWebView.h/.cpp`.
//!
//! An ActiveX-hosted IE (`IWebBrowser2`) dialog used for the in-game browser.
//! The original is a `CAxDialogImpl` implementing `IDocHostUIHandler` +
//! `IDispatch`, hooking `DWebBrowserEvents`. No anti-cheat code; navigation is
//! gated by `RBX::Http::trustCheckBrowser`.
//!
//! In idiomatic Rust the COM surface is expressed with the `windows` crate's
//! `#[implement]`; here we keep the behavioural shape and route trust checks
//! and lifetime through plain methods. Wiring to a live `IWebBrowser2` host
//! requires the dialog template `IDD_RBXWEBVIEW` (in `WindowsClient.rc`).

#![allow(dead_code)]

use std::sync::{Arc, Weak};

use windows::Win32::Foundation::HWND;

use crate::rbx::{self, Game};

const ROBLOX_BROWSER_FLAGS: u32 =
    // DOCHOSTUIFLAG_DISABLE_HELP_MENU | ENABLE_FORMS_AUTOCOMPLETE | THEME |
    // DISABLE_SCRIPT_INACTIVE | LOCAL_MACHINE_ACCESS_CHECK |
    // DISABLE_UNTRUSTEDPROTOCOL | NO3DBORDER
    0x0000_0001 | 0x0001_0000 | 0x0004_0000 | 0x0000_0010 | 0x0010_0000 | 0x0040_0000 | 0x0000_0004;

/// `RbxWebView`.
pub struct RbxWebView {
    url: String,
    game: Weak<dyn Game>,
    dialog_active: bool,
    hwnd: HWND,
}

impl RbxWebView {
    pub fn new(url: String, game: Arc<dyn Game>) -> Self {
        Self { url, game: Arc::downgrade(&game), dialog_active: false, hwnd: HWND::default() }
    }

    /// `OnInitDialog` — center, set icon, wire browser events, navigate.
    pub fn on_init_dialog(&mut self) -> bool {
        // GetDlgControl(IDC_RBXEXPLORER, IWebBrowserApp) + FindConnectionPoint +
        // Advise(DWebBrowserEvents/2). Set custom user-agent via
        // UrlMkSetSessionOption, then pWebBrowser->Navigate(url).
        self.dialog_active = true;
        true
    }

    /// `BeforeNavigate2` — allow only trusted browser URLs.
    pub fn before_navigate2(&self, url: &str) -> bool {
        rbx::http::Http::trust_check_browser(url)
    }

    /// `IDocHostUIHandler::GetHostInfo` flags.
    pub fn host_info_flags(&self) -> u32 {
        ROBLOX_BROWSER_FLAGS
    }

    /// `WindowClosing` — end the dialog and signal `urlWindowClosed`.
    pub fn close_dialog(&mut self) {
        if !self.dialog_active {
            return;
        }
        self.dialog_active = false;
        if let Some(game) = self.game.upgrade() {
            if let Some(dm) = game.get_data_model() {
                if let Some(gs) = dm.find_gui_service() {
                    gs.url_window_closed();
                }
            }
        }
    }
}
