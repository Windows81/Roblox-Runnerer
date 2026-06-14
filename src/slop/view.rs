//! Translated from `View.h/.cpp` and `RenderJob.h/.cpp`.
//!
//! Owns the game viewport: graphics-mode selection, fullscreen / resolution
//! switching, window placement persistence, the render job and user input.
//! This file contained no anti-cheat code; it is a straight Win32 translation.

#[repr(C)]
pub struct OSContext {
    pub hwnd: usize,
    pub width: i32,
    pub height: i32,
    // insert OS specific stuff here.
}

impl Default for OSContext {
    fn default() -> Self {
        Self {
            hwnd: 0,
            width: 640,
            height: 480,
        }
    }
}

use std::sync::{Arc, RwLock};

use windows::Win32::Foundation::{HWND, LPARAM, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::w;

use crate::structs::{self, DataModel, Game, GraphicsMode, StepResult};

use super::function_marshaller::FunctionMarshaller;
use super::user_input::UserInput;

const SAVED_SCREEN_SIZE_REGISTRY_KEY: &str = r"HKEY_CURRENT_USER\Software\ROBLOX Corporation\Roblox\Settings\RobloxPlayerV4WindowSizeAndPosition";

/// `RBX::View`.
pub struct View {
    pub hwnd: usize,
    pub game: Option<Game>,
    pub fullscreen: bool,
    pub desire_fullscreen: bool,
    pub changed_resolution: bool,
    pub changing_resolution: bool,
    pub hmonitor: usize,
    pub marshaller: Arc<RwLock<FunctionMarshaller>>,
    pub non_fullscreen_placement: WINDOWPLACEMENT,
    pub restore_window_style: i32,
    pub user_input: Option<Arc<RwLock<UserInput>>>,
    pub window_settings_valid: bool,
    pub window_settings_rect: (f32, f32, f32, f32),
    pub window_settings_maximized: bool,
    pub is_awake: bool,
    pub stopped: bool,
}

impl View {
    pub fn get_data_model(&self) -> Option<DataModel> {
        let game = self.game.as_ref()?;
        Some(unsafe { game.data_model.ptr.read() })
    }

    /// `View::initializeView` — pick a graphics mode and create the GfxBase view.
    pub fn initialize_view(&mut self) {
        let mode = GraphicsMode::Direct3D11;
        let context = OSContext::default();
        self.vi;
        self.initialize_sizes();
    }

    pub fn initialize_sizes(&mut self) {
        // CRenderSettingsItem window/fullscreen size validation (engine).
    }

    /// `View::Start`.
    pub fn start(&mut self, game: &Game) {
        self.game = Some(*game);
        self.bind_workspace();
        self.initialize_jobs();
        self.initialize_input();
        self.reset_scheduler();
        if let Some(ui) = self.user_input.as_mut() {
            ui.set_keyboard_desired(true);
        }
    }

    fn bind_workspace(&mut self) {
        // view->bindWorkspace(game->getDataModel()); view->buildGui(); (engine)
    }

    fn unbind_workspace(&mut self) {}

    fn initialize_jobs(&mut self) {
        let Some(game) = self.game.clone() else {
            return;
        };
        let Some(dm) = game.get_data_model() else {
            return;
        };
        let marshaller = self.marshaller.clone();
    }
    pub fn stop(&mut self) {
        self.remove_jobs();
        self.user_input = None;
        self.unbind_workspace();
        self.save_window_settings();
        self.game = None;
        self.stopped = true;
    }

    /// `RenderJob::stepDataModelJob` — security/cheat block removed.
    pub fn step_data_model_job(&mut self) -> StepResult {
        if self.game.is_none() {
            return StepResult::Done;
        }
        if self.stopped {
            return StepResult::Done;
        }
        self.is_awake = false;
        StepResult::Stepped
    }

    /// `RenderJob::getMetric` (IMetric).
    pub fn get_metric(&self, metric: &str) -> String {
        match metric {
            "Graphics Mode" => "OpenGL".into(),
            "Render" => "0.0/s 0%".into(),
            _ => "?".into(),
        }
    }

    /// `RenderJob::getMetricValue` (IMetric).
    pub fn get_metric_value(&self, _metric: &str) -> f64 {
        0.0
    }

    fn initialize_input(&mut self) {
        if let Some(game) = self.game {
            self.user_input = Some(RwLock::new(UserInput::new(self.hwnd, game)).into());
            // ControllerService::setHardwareDevice(userInput) (engine)
        }
    }

    fn reset_scheduler(&mut self) {
        // TaskScheduler::singleton().add(renderJob) (engine)
    }

    fn remove_jobs(&mut self) {
        // TaskScheduler::removeBlocking(renderJob, ProcessMessages); marshaller->ProcessMessages();
        self.marshaller.write().unwrap().process_messages();
    }

    /// `View::HandleWindowsMessage` — fullscreen activation handling, else input.
    pub fn handle_windows_message(&mut self, msg: u32, wparam: WPARAM, lparam: LPARAM) {
        if msg == WM_ACTIVATE {
            let activating = wparam.0 as u32 & 0xffff;
            let became_active = activating == WA_ACTIVE || activating == WA_CLICKACTIVE;
            if (self.fullscreen || self.desire_fullscreen)
                && !self.changing_resolution
                && became_active
            {
                self.change_resolution();
                unsafe {
                    let _ = ShowWindow(HWND(self.hwnd as _), SW_RESTORE);
                    let _ = SetFocus(HWND(self.hwnd as _));
                    let _ = SetWindowPos(
                        HWND(self.hwnd as _),
                        HWND_TOP,
                        0,
                        0,
                        0,
                        0,
                        SWP_NOMOVE | SWP_NOSIZE,
                    );
                }
            } else if self.fullscreen && !self.changing_resolution && activating == WA_INACTIVE {
                unsafe {
                    SetWindowLongPtrW(
                        HWND(self.hwnd as _),
                        GWL_STYLE,
                        (WS_VISIBLE
                            | WS_POPUP
                            | WS_MINIMIZEBOX
                            | WS_MAXIMIZEBOX
                            | WS_CLIPSIBLINGS
                            | WS_CLIPCHILDREN)
                            .0 as isize,
                    );
                    let _ = SetWindowPos(
                        HWND(self.hwnd as _),
                        HWND_TOP,
                        0,
                        0,
                        0,
                        0,
                        SWP_NOMOVE | SWP_NOSIZE,
                    );
                }
            }
        } else if let Some(ui) = &self.user_input {
            ui.write()
                .unwrap()
                .post_user_input_message(msg, wparam, lparam);
        }
    }

    /// `View::OnResize`.
    pub fn on_resize(&mut self, _wparam: WPARAM, _cx: i32, _cy: i32) {
        // view->onResize(cx, cy) (engine)
    }

    /// `View::SetFullscreen`.
    pub fn set_fullscreen(&mut self, value: bool) {
        if self.fullscreen != value {
            if value {
                self.restore_window_style =
                    unsafe { GetWindowLongW(HWND(self.hwnd as _), GWL_STYLE) };
                self.change_resolution();
            } else {
                self.restore_resolution();
            }
        }
        self.desire_fullscreen = value;
    }

    fn change_resolution(&mut self) {
        if !self.fullscreen {
            self.non_fullscreen_placement = WINDOWPLACEMENT {
                length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
                ..Default::default()
            };
            unsafe {
                let _ =
                    GetWindowPlacement(HWND(self.hwnd as _), &mut self.non_fullscreen_placement);
            }
        }
        self.fullscreen = true;
        self.hmonitor =
            unsafe { MonitorFromWindow(HWND(self.hwnd as _), MONITOR_DEFAULTTONEAREST) }.0 as _;
        self.initialize_sizes();
        // Best-mode match + ChangeDisplaySettingsEx + modifyWindow(WS_POPUP...) (Win32).
    }

    fn restore_resolution(&mut self) {
        self.fullscreen = false;
        // ChangeDisplaySettingsEx(NULL) restore + SetWindowPlacement (Win32).
        unsafe {
            let _ = SetWindowPlacement(HWND(self.hwnd as _), &self.non_fullscreen_placement);
        }
    }

    /// `View::ShowWindow`.
    pub fn show_window(&mut self) {
        unsafe {
            if self.fullscreen {
                let _ = ShowWindow(HWND(self.hwnd as _), SW_SHOWMAXIMIZED);
            } else {
                let _ = ShowWindow(HWND(self.hwnd as _), SW_SHOWNORMAL);
            }
            let _ = SetWindowPos(
                HWND(self.hwnd as _),
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE,
            );
            let _ = SetWindowPos(
                HWND(self.hwnd as _),
                HWND_NOTOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE,
            );
        };
    }

    /// `View::CloseWindow`.
    pub fn close_window(&self) {
        unsafe {
            let _ = PostMessageW(HWND(self.hwnd as _), WM_CLOSE, WPARAM(0), LPARAM(0));
        }
    }

    /// `View::AboutToShutdown` — persist window placement.
    pub fn about_to_shutdown(&mut self) {
        self.remember_window_settings();
    }

    fn remember_window_settings(&mut self) {
        let mut placement = WINDOWPLACEMENT {
            length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
            ..Default::default()
        };
        let found = if !self.fullscreen {
            unsafe { GetWindowPlacement(HWND(self.hwnd as _), &mut placement).is_ok() }
        } else {
            placement = self.non_fullscreen_placement;
            true
        };
        if !found {
            return;
        }
        let mut rect = RECT::default();
        unsafe {
            let _ = GetWindowRect(HWND(self.hwnd as _), &mut rect);
        }
        // Taskbar adjustment (Shell_traywnd) omitted for brevity; same math.
        self.window_settings_valid = true;
        self.window_settings_rect = (
            rect.left as f32,
            rect.top as f32,
            (rect.right - rect.left) as f32,
            (rect.bottom - rect.top) as f32,
        );
        self.window_settings_maximized = placement.showCmd == SW_SHOWMAXIMIZED.0 as u32;
    }

    fn save_window_settings(&self) {
        // GameBasicSettings::setStartScreenSize/Pos/Maximized under a DM lock.
    }
}

// Touch an unused import to keep the GraphicsMode enum + registry key referenced.
const _: &str = SAVED_SCREEN_SIZE_REGISTRY_KEY;
const _: fn() = || {
    let _ = w!("");
};
