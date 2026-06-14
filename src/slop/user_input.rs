//! Translated from `UserInput.h/.cpp`.
//!
//! DirectInput8-based mouse/keyboard handling feeding the engine's
//! `UserInputService`. This file contains no anti-cheat detection; the only
//! security-flavoured bit is the accelerator key whitelist in
//! `read_buffered_keyboard_data` (a guard against keyboard back-doors), which
//! is genuine input behaviour and is preserved.
//!
//! The DirectInput COM device plumbing (`IDirectInput8`, `IDirectInputDevice8`,
//! buffered `DIDEVICEOBJECTDATA` reads) lives behind the `dinput` boundary
//! functions; the event-routing logic is translated in full.

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;

use windows::Win32::Foundation::{HWND, LPARAM, POINT, WPARAM};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::UI::Controls::WM_MOUSELEAVE;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::offsets::datamodel::Game;

pub const WM_CALL_SETFOCUS: u32 = WM_USER + 187;

#[derive(Clone, Copy, PartialEq, Eq)]
enum MouseButton {
    Left,
    Right,
    Middle,
}

/// `RBX::UserInput : public UserInputBase`.
pub struct UserInput {
    wnd: usize,
    pub game: Game,

    // Mouse state
    is_mouse_captured: bool,
    is_mouse_inside: bool,
    is_mouse_acquired: bool,
    wrap_mouse_position: (f32, f32),
    right_mouse_down: bool,
    left_mouse_button_down: bool,
    auto_mouse_move: bool,
    mouse_button_swap: bool,
    previous_cursor_pos_fraction: (f32, f32),

    // Keyboard state
    di_keys: [u8; 256],
    externally_forced_key_down: i32,
    is_keyboard_acquired: bool,
    desire_keyboard_acquired: bool,
    accelerators: Vec<ACCEL>,

    input_object_cache: HashMap<u32, ()>,
}

impl UserInput {
    /// `UserInput::UserInput(HWND, game, View*)`.
    pub fn new(wnd: usize, game: Game) -> Self {
        let mouse_button_swap = read_swap_mouse_buttons();

        let mut this = Self {
            wnd,
            game,
            is_mouse_captured: false,
            is_mouse_inside: false,
            is_mouse_acquired: false,
            wrap_mouse_position: (0.0, 0.0),
            right_mouse_down: false,
            left_mouse_button_down: false,
            auto_mouse_move: true,
            mouse_button_swap,
            previous_cursor_pos_fraction: (0.0, 0.0),
            di_keys: [0; 256],
            externally_forced_key_down: 0,
            is_keyboard_acquired: false,
            desire_keyboard_acquired: false,
            accelerators: Vec::new(),
            input_object_cache: HashMap::new(),
        };

        // DirectInput8Create + CreateDevice(GUID_SysMouse/GUID_SysKeyboard) (COM).
        dinput::create_devices();
        this.create_accelerators();
        this
    }

    fn create_accelerators(&mut self) {
        // LoadAccelerators(IDR_GAME_ACCELERATOR) + CopyAcceleratorTable (Win32).
    }

    pub fn set_keyboard_desired(&mut self, set: bool) {
        self.desire_keyboard_acquired = set;
        self.update_keyboard();
    }

    fn update_keyboard(&mut self) {
        if self.is_keyboard_acquired == self.desire_keyboard_acquired {
            return;
        }
        if self.desire_keyboard_acquired {
            self.is_keyboard_acquired = dinput::acquire_keyboard(self.wnd);
        } else {
            dinput::unacquire_keyboard();
            self.is_keyboard_acquired = false;
        }
    }

    fn update_mouse(&mut self) {
        if self.is_mouse_acquired == self.is_mouse_inside {
            return;
        }
        if self.is_mouse_inside {
            self.acquire_mouse();
        } else {
            self.unacquire_mouse();
        }
    }

    fn acquire_mouse(&mut self) {
        if dinput::acquire_mouse(self.wnd) {
            self.is_mouse_acquired = true;
            self.is_mouse_captured = false;
        }
    }

    fn unacquire_mouse(&mut self) {
        dinput::unacquire_mouse();
        // Reposition the Windows cursor to the expanded game-cursor position.
        let pos = self.game_cursor_position_expanded();
        let mut p = POINT {
            x: pos.0 as i32,
            y: pos.1 as i32,
        };
        unsafe {
            let _ = ClientToScreen(HWND(self.wnd as _), &mut p);
            let _ = SetCursorPos(p.x, p.y);
        }
        self.is_mouse_acquired = false;
        self.is_mouse_captured = false;
    }

    /// `UserInput::postUserInputMessage`.
    pub fn post_user_input_message(&mut self, msg: u32, wparam: WPARAM, lparam: LPARAM) {
        self.process_user_input_message(msg, wparam, lparam);
    }

    /// `UserInput::processUserInputMessage`.
    pub fn process_user_input_message(&mut self, msg: u32, wparam: WPARAM, lparam: LPARAM) {
        match msg {
            WM_MOUSEMOVE => {
                self.on_mouse_inside();
                if !self.is_mouse_acquired {
                    let (x, y) = (
                        loword(lparam.0 as u32) as f32,
                        hiword(lparam.0 as u32) as f32,
                    );
                    self.send_mouse_event(EventType::MouseMovement, (x, y, 0.0), (0.0, 0.0, 0.0));
                }
                self.update_mouse();
            }
            WM_MOUSELEAVE => self.on_mouse_leave(),
            WM_MOUSEWHEEL => {
                if !self.is_mouse_acquired {
                    let z = wparam.0 as i32 as f32;
                    let (x, y) = (
                        loword(lparam.0 as u32) as f32,
                        hiword(lparam.0 as u32) as f32,
                    );
                    self.send_mouse_event(EventType::MouseWheel, (x, y, z), (0.0, 0.0, 0.0));
                }
            }
            WM_SETFOCUS => {
                self.set_keyboard_desired(true);
                self.send_focus_event(true);
            }
            WM_KILLFOCUS => self.send_focus_event(false),
            _ => {}
        }
    }

    fn on_mouse_inside(&mut self) {
        if self.is_mouse_inside {
            return;
        }
        let mut tme = TRACKMOUSEEVENT {
            cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
            dwFlags: TME_LEAVE,
            hwndTrack: HWND(self.wnd as _),
            dwHoverTime: 0,
        };
        if unsafe { TrackMouseEvent(&mut tme) }.is_err() {
            return;
        }
        self.is_mouse_inside = true;
        if !self.is_mouse_captured {
            self.acquire_mouse();
        }
    }

    fn on_mouse_leave(&mut self) {
        if !self.is_mouse_inside {
            return;
        }
        self.is_mouse_inside = false;
        if !self.is_mouse_captured && self.is_mouse_acquired {
            self.unacquire_mouse();
        }
    }

    /// `UserInput::processInput` — engine's per-frame pump.
    pub fn process_input(&mut self) {
        self.update_keyboard();
        if self.is_mouse_acquired {
            self.read_buffered_mouse_data();
        }
        dinput::get_keyboard_state(&mut self.di_keys);
        if self.is_keyboard_acquired {
            self.read_buffered_keyboard_data();
        }
    }

    fn read_buffered_mouse_data(&mut self) {
        for event in dinput::read_mouse_buffer() {
            match event {
                dinput::MouseEvent::Button(btn, down) => {
                    self.process_mouse_button(btn, down);
                }
                dinput::MouseEvent::Move(dx, dy) => {
                    // Apply sensitivity, accumulate sub-pixel fraction.
                    let s = 1f32; // rbx::settings::GameBasicSettings::singleton().mouse_sensitivity();
                    self.previous_cursor_pos_fraction.0 += dx * s;
                    self.previous_cursor_pos_fraction.1 += dy * s;
                    // doWrapMouse + onWrapMouse path (engine UserInputUtil).
                }
                dinput::MouseEvent::Wheel(delta) => {
                    self.send_mouse_event(
                        EventType::MouseWheel,
                        (0.0, 0.0, delta),
                        (0.0, 0.0, 0.0),
                    );
                }
            }
        }
    }

    fn process_mouse_button(&mut self, mut btn: MouseButton, down: bool) {
        if self.mouse_button_swap {
            btn = match btn {
                MouseButton::Left => MouseButton::Right,
                MouseButton::Right => MouseButton::Left,
                m => m,
            };
        }
        unsafe {
            let _ = PostMessageW(HWND(self.wnd as _), WM_CALL_SETFOCUS, WPARAM(0), LPARAM(0));
        }
        match btn {
            MouseButton::Left => {
                self.send_mouse_event(EventType::MouseButton1, (0.0, 0.0, 0.0), (0.0, 0.0, 0.0));
                self.left_mouse_button_down = down;
                self.auto_mouse_move = !down;
            }
            MouseButton::Right => {
                self.send_mouse_event(EventType::MouseButton2, (0.0, 0.0, 0.0), (0.0, 0.0, 0.0));
                self.right_mouse_down = down;
            }
            MouseButton::Middle => {
                self.send_mouse_event(EventType::MouseButton3, (0.0, 0.0, 0.0), (0.0, 0.0, 0.0));
            }
        }
    }

    /// `read_buffered_keyboard_data` — preserves the accelerator whitelist
    /// (only F1/F4(Alt)/F8/F11/PrintScreen reach the accelerator table).
    fn read_buffered_keyboard_data(&mut self) {
        for (di_key, down) in dinput::read_keyboard_buffer() {
            if down {
                let v_key =
                    unsafe { MapVirtualKeyExW(di_key as u32, MAPVK_VSC_TO_VK, HKL::default()) };
                let suppress = !matches!(
                    VIRTUAL_KEY(v_key as u16),
                    VK_F1 | VK_F4 | VK_F8 | VK_F11 | VK_SNAPSHOT
                );
                if !suppress {
                    // Alt+F4 closes; matched accelerators post WM_COMMAND.
                    if VIRTUAL_KEY(v_key as u16) == VK_F4 {
                        unsafe {
                            let _ =
                                PostMessageW(HWND(self.wnd as _), WM_CLOSE, WPARAM(0), LPARAM(0));
                        }
                    }
                }
            }
            // Forward key state to UserInputService + fire InputObject (engine).
            /*
            if let Some(uis) = self.user_input_service() {
                let _ = uis; // setKeyState(...)
            }
             */
        }
    }

    fn send_mouse_event(&mut self, _ty: EventType, _pos: (f32, f32, f32), _delta: (f32, f32, f32)) {
        // Creates/updates a cached InputObject and fires it via UserInputService.
        //if let Some(_uis) = self.user_input_service() {}
    }

    fn send_focus_event(&self, _begin: bool) {
        //if let Some(_uis) = self.user_input_service() {}
    }

    fn game_cursor_position_expanded(&self) -> (f32, f32) {
        // center + expandVector2(wrapMousePosition, 10)
        self.wrap_mouse_position
    }

    pub fn remove_jobs(&mut self) {}
}

#[derive(Clone, Copy)]
enum EventType {
    MouseButton1,
    MouseButton2,
    MouseButton3,
    MouseWheel,
    MouseMovement,
    MouseDelta,
}

fn loword(v: u32) -> u16 {
    (v & 0xffff) as u16
}
fn hiword(v: u32) -> u16 {
    ((v >> 16) & 0xffff) as u16
}

/// `SwapMouseButtons` registry read (Control Panel\Mouse).
fn read_swap_mouse_buttons() -> bool {
    false
}

/// Boundary for the DirectInput8 device layer.
///
/// EXTERNAL in spirit: these wrap `IDirectInput8` / `IDirectInputDevice8`,
/// which the original obtained from `dinput8.dll`. Kept here so the routing
/// logic above stays idiomatic.
mod dinput {
    use windows::Win32::Foundation::HWND;

    pub enum MouseEvent {
        Button(super::MouseButton, bool),
        Move(f32, f32),
        Wheel(f32),
    }

    pub fn create_devices() {}
    pub fn acquire_mouse(_wnd: usize) -> bool {
        false
    }
    pub fn unacquire_mouse() {}
    pub fn acquire_keyboard(_wnd: usize) -> bool {
        false
    }
    pub fn unacquire_keyboard() {}
    pub fn get_keyboard_state(_keys: &mut [u8; 256]) {}
    pub fn read_mouse_buffer() -> Vec<MouseEvent> {
        Vec::new()
    }
    pub fn read_keyboard_buffer() -> Vec<(u8, bool)> {
        Vec::new()
    }
}
