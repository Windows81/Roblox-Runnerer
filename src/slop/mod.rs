//! Translated from `main.cpp` — entry point, window class, and `WndProc`.
//!
//! ## Anti-cheat / VMProtect removed
//! The original `_tWinMain` ended with
//! `VirtualProtect(RBX::Security::rbxVmpBase, rbxVmpSize, PAGE_EXECUTE_READWRITE)`
//! to restore the VMProtect sections before shutdown. That call (and the
//! `RBX::Security` globals it referenced) is removed.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]

mod app;
mod document;
mod function_marshaller;
mod game_verbs;
mod rbx;
mod render_job;
mod resource;
mod user_input;
mod view;

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::UpdateWindow;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::WM_MOUSELEAVE;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::{PCWSTR, w};

use resource::*;

use app::Application;

const WINDOW_WIDTH: i32 = 800;
const WINDOW_HEIGHT: i32 = 600;
static mut APP_PTR: *mut Application = std::ptr::null_mut();

/// `WndProc` — main window message handler.
unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let app = || -> &mut Application { unsafe { &mut *APP_PTR } };
    match message {
        WM_TIMER => {
            // LogManager hang-detection heartbeat (NotifyFGThreadAlive).
            LRESULT(0)
        }
        WM_COMMAND => {
            match (wparam.0 & 0xffff) as u32 {
                ID_UPLOADSESSIONLOGS => app().upload_session_logs(),
                ID_LOADWIKI => app().on_help(),
                _ => {}
            }
            LRESULT(0)
        }
        WM_GETMINMAXINFO => {
            Application::on_get_min_max_info(lparam.0 as *mut MINMAXINFO);
            LRESULT(0)
        }
        WM_KEYDOWN | WM_MOUSEMOVE | WM_MOUSELEAVE | WM_MOUSEWHEEL | WM_SETFOCUS | WM_KILLFOCUS
        | WM_ACTIVATE | WM_ACTIVATEAPP | WM_CHAR | WM_INPUT => {
            app().handle_windows_message(message, wparam, lparam);
            LRESULT(0)
        }
        WM_DESTROY => {
            app().about_to_shutdown();
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        WM_SIZE => {
            app().on_resize(
                wparam,
                loword(lparam.0 as u32) as i32,
                hiword(lparam.0 as u32) as i32,
            );
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

fn register_window_class(hinstance: windows::Win32::Foundation::HINSTANCE) -> u16 {
    let wcex = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wnd_proc),
        hInstance: hinstance,
        hIcon: unsafe {
            LoadIconW(hinstance, make_int_resource(IDI_WINDOW_ICON)).unwrap_or_default()
        },
        hCursor: unsafe { LoadCursorW(None, IDC_ARROW).unwrap_or_default() },
        lpszClassName: w!("RobloxPlayerWindow"),
        lpszMenuName: make_int_resource(IDC_WINDOWSCLIENT),
        hIconSm: unsafe {
            LoadIconW(hinstance, make_int_resource(IDI_WINDOW_ICON)).unwrap_or_default()
        },
        ..Default::default()
    };
    unsafe { RegisterClassExW(&wcex) }
}

fn main() -> windows::core::Result<()> {
    // This binary was compiled with SSE2; G3D::System::hasSSE2() guard omitted
    // (all supported x86-64 targets have SSE2).
    let hinstance = unsafe { GetModuleHandleW(None)? };

    let mut application = Application::new();
    unsafe { APP_PTR = &mut application as *mut Application };

    // CComModule comModule — ATL COM apartment for ActiveX IWebBrowser2 hosting.
    unsafe {
        let _ = windows::Win32::System::Com::CoInitializeEx(
            None,
            windows::Win32::System::Com::COINIT_APARTMENTTHREADED,
        );
    }

    if !application.load_app_settings(hinstance.0 as isize) {
        return Ok(());
    }
    let cmd_line = command_line_tail();
    if !application.parse_arguments(&cmd_line) {
        return Ok(());
    }

    // Client settings fetch/apply (FetchClientSettingsData) — engine.

    register_window_class(hinstance.into());

    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("RobloxPlayerWindow"),
            w!("ROBLOX"),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            None,
            None,
            hinstance,
            None,
        )?
    };
    if hwnd.is_invalid() {
        return Ok(());
    }

    match application.initialize(hwnd, hinstance.0 as isize) {
        Ok(true) => {}
        Ok(false) => return Ok(()),
        Err(e) => {
            let msg: Vec<u16> = format!("{e}\0").encode_utf16().collect();
            unsafe {
                MessageBoxW(hwnd, PCWSTR(msg.as_ptr()), w!("ROBLOX"), MB_OK);
            }
            application.about_to_shutdown();
            application.shutdown();
            return Ok(());
        }
    }

    // Keep-alive timer for the WM_TIMER hang-detection heartbeat.
    unsafe {
        SetTimer(hwnd, 0, 10 * 1000, None);
        let _ = ShowWindow(hwnd, SW_HIDE);
        let _ = UpdateWindow(hwnd);
    }

    // Main message loop.
    let mut msg = MSG::default();
    unsafe {
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    // [removed] VirtualProtect(RBX::Security::rbxVmpBase, rbxVmpSize, PAGE_EXECUTE_READWRITE)

    application.shutdown();
    unsafe { APP_PTR = std::ptr::null_mut() };
    Ok(())
}

fn make_int_resource(id: u32) -> PCWSTR {
    PCWSTR(id as u16 as usize as *const u16)
}

fn loword(v: u32) -> u16 {
    (v & 0xffff) as u16
}
fn hiword(v: u32) -> u16 {
    ((v >> 16) & 0xffff) as u16
}

/// The command-line tail (everything after the program name), mirroring the
/// `lpCmdLine` that `_tWinMain` receives.
fn command_line_tail() -> String {
    std::env::args().skip(1).collect::<Vec<_>>().join(" ")
}
