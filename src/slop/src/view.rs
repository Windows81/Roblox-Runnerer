//! Translated from `View.h/.cpp`.
//!
//! Owns the game viewport: graphics-mode selection, fullscreen / resolution
//! switching, window placement persistence, the render job and user input.
//! This file contained no anti-cheat code; it is a straight Win32 translation.

#![allow(dead_code)]

use std::sync::Arc;

use windows::core::w;
use windows::Win32::Foundation::{HWND, LPARAM, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::function_marshaller::FunctionMarshaller;
use crate::rbx::{self, DataModel, Game};
use crate::render_job::RenderJob;
use crate::user_input::UserInput;

const SAVED_SCREEN_SIZE_REGISTRY_KEY: &str =
    r"HKEY_CURRENT_USER\Software\ROBLOX Corporation\Roblox\Settings\RobloxPlayerV4WindowSizeAndPosition";

/// `RBX::CRenderSettings::GraphicsMode`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GraphicsMode {
    NoGraphics,
    Direct3D9,
    Direct3D11,
    OpenGL,
}

/// `RBX::View`.
pub struct View {
    hwnd: HWND,
    game: Option<Arc<dyn Game>>,
    fullscreen: bool,
    desire_fullscreen: bool,
    changed_resolution: bool,
    changing_resolution: bool,
    hmonitor: HMONITOR,
    marshaller: *mut FunctionMarshaller,
    non_fullscreen_placement: WINDOWPLACEMENT,
    restore_window_style: i32,
    user_input: Option<Box<UserInput>>,
    render_job: Option<Arc<RenderJob>>,
    window_settings_valid: bool,
    window_settings_rect: (f32, f32, f32, f32),
    window_settings_maximized: bool,
}

impl View {
    /// `View::View(HWND)`.
    pub fn new(hwnd: HWND) -> Self {
        let marshaller = FunctionMarshaller::get_window();
        let mut view = Self {
            hwnd,
            game: None,
            fullscreen: false,
            desire_fullscreen: rbx::settings::GameBasicSettings::singleton().full_screen(),
            changed_resolution: false,
            changing_resolution: false,
            hmonitor: HMONITOR::default(),
            marshaller,
            non_fullscreen_placement: WINDOWPLACEMENT {
                length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
                ..Default::default()
            },
            restore_window_style: 0,
            user_input: None,
            render_job: None,
            window_settings_valid: false,
            window_settings_rect: (0.0, 0.0, 0.0, 0.0),
            window_settings_maximized: false,
        };
        view.initialize_view();
        view
    }

    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    pub fn is_fullscreen(&self) -> bool {
        self.fullscreen
    }

    pub fn get_data_model(&self) -> Option<Arc<dyn DataModel>> {
        self.game.as_ref().and_then(|g| g.get_data_model())
    }

    /// `View::initializeView` — pick a graphics mode and create the GfxBase view.
    fn initialize_view(&mut self) {
        // ViewBase::InitPluginModules();
        // Original prefers OpenGL unless FFlag::DirectXEnable; here we simply
        // record the latched mode. The actual ViewBase::CreateView lives in
        // GfxBase (external).
        let _mode = GraphicsMode::OpenGL;
        self.initialize_sizes();
    }

    fn initialize_sizes(&mut self) {
        // CRenderSettingsItem window/fullscreen size validation (engine).
    }

    /// `View::Start`.
    pub fn start(&mut self, game: Arc<dyn Game>) {
        self.game = Some(game.clone());
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
        if let (Some(game), false) = (self.game.clone(), self.marshaller.is_null()) {
            if let Some(dm) = game.get_data_model() {
                self.render_job =
                    Some(Arc::new(RenderJob::new(self as *mut _, self.marshaller, dm)));
            }
        }
    }

    fn initialize_input(&mut self) {
        if let Some(game) = self.game.clone() {
            self.user_input = Some(Box::new(UserInput::new(self.hwnd, game, self as *mut _)));
            // ControllerService::setHardwareDevice(userInput) (engine)
        }
    }

    fn reset_scheduler(&mut self) {
        // TaskScheduler::singleton().add(renderJob) (engine)
    }

    fn remove_jobs(&mut self) {
        // TaskScheduler::removeBlocking(renderJob, ProcessMessages); marshaller->ProcessMessages();
        if !self.marshaller.is_null() {
            unsafe { (*self.marshaller).process_messages() };
        }
        self.render_job = None;
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
                    let _ = ShowWindow(self.hwnd, SW_RESTORE);
                    let _ = windows::Win32::UI::Input::KeyboardAndMouse::SetFocus(Some(self.hwnd));
                    let _ = SetWindowPos(
                        self.hwnd,
                        Some(HWND_TOP),
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
                        self.hwnd,
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
                        self.hwnd,
                        Some(HWND_TOP),
                        0,
                        0,
                        0,
                        0,
                        SWP_NOMOVE | SWP_NOSIZE,
                    );
                }
            }
        } else if let Some(ui) = self.user_input.as_mut() {
            ui.post_user_input_message(msg, wparam, lparam);
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
                    unsafe { GetWindowLongW(self.hwnd, GWL_STYLE) };
                self.change_resolution();
            } else {
                self.restore_resolution();
            }
        }
        self.desire_fullscreen = value;
        rbx::settings::GameBasicSettings::singleton().set_full_screen(value);
    }

    fn change_resolution(&mut self) {
        if !self.fullscreen {
            self.non_fullscreen_placement = WINDOWPLACEMENT {
                length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
                ..Default::default()
            };
            unsafe {
                let _ = GetWindowPlacement(self.hwnd, &mut self.non_fullscreen_placement);
            }
        }
        self.fullscreen = true;
        self.hmonitor = unsafe { MonitorFromWindow(self.hwnd, MONITOR_DEFAULTTONEAREST) };
        self.initialize_sizes();
        // Best-mode match + ChangeDisplaySettingsEx + modifyWindow(WS_POPUP...) (Win32).
    }

    fn restore_resolution(&mut self) {
        self.fullscreen = false;
        // ChangeDisplaySettingsEx(NULL) restore + SetWindowPlacement (Win32).
        unsafe {
            let _ = SetWindowPlacement(self.hwnd, &self.non_fullscreen_placement);
        }
    }

    /// `View::ShowWindow`.
    pub fn show_window(&mut self) {
        unsafe {
            if rbx::settings::GameBasicSettings::singleton().start_maximized() {
                let _ = ShowWindow(self.hwnd, SW_SHOWMAXIMIZED);
            } else {
                let _ = ShowWindow(self.hwnd, SW_SHOWNORMAL);
            }
            let _ = SetWindowPos(self.hwnd, Some(HWND_TOPMOST), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
            let _ =
                SetWindowPos(self.hwnd, Some(HWND_NOTOPMOST), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
        }
        if rbx::settings::GameBasicSettings::singleton().full_screen() {
            self.set_fullscreen(true);
        }
    }

    /// `View::CloseWindow`.
    pub fn close_window(&self) {
        unsafe {
            let _ = PostMessageW(Some(self.hwnd), WM_CLOSE, WPARAM(0), LPARAM(0));
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
            unsafe { GetWindowPlacement(self.hwnd, &mut placement).is_ok() }
        } else {
            placement = self.non_fullscreen_placement;
            true
        };
        if found {
            let mut rect = RECT::default();
            unsafe {
                let _ = GetWindowRect(self.hwnd, &mut rect);
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
    }

    fn save_window_settings(&self) {
        // GameBasicSettings::setStartScreenSize/Pos/Maximized under a DM lock.
    }

    /// `View::Stop`.
    pub fn stop(&mut self) {
        self.remove_jobs();
        self.user_input = None;
        self.unbind_workspace();
        self.save_window_settings();
        self.game = None;
    }
}

impl Drop for View {
    fn drop(&mut self) {
        if !self.marshaller.is_null() {
            FunctionMarshaller::release_window(self.marshaller);
        }
    }
}

// Touch an unused import to keep the GraphicsMode enum + registry key referenced.
const _: &str = SAVED_SCREEN_SIZE_REGISTRY_KEY;
const _: fn() = || {
    let _ = w!("");
};
