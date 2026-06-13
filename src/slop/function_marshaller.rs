//! Translated from `FunctionMarshaller.h/.cpp`.

#![allow(dead_code)]

use std::sync::{Arc, Mutex};

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::{PCWSTR, w};

const WM_EVENT: u32 = WM_USER + 101;
const WM_ASYNCEVENT: u32 = WM_USER + 102;

type Job = extern "C" fn();

/// One marshaller per thread, mirroring the C++ static `windows` map.
pub struct FunctionMarshaller {
    hwnd: usize,
    thread_id: u32,
    async_calls: Arc<Mutex<Vec<Job>>>,
    posted_async: Arc<std::sync::atomic::AtomicBool>,
    ref_count: usize,
}

impl Drop for FunctionMarshaller {
    /// `FunctionMarshaller::ReleaseWindow()`.
    fn drop(&mut self) {
        unsafe { DestroyWindow(HWND(self.hwnd as _)) };
    }
}

impl FunctionMarshaller {
    pub fn new() -> Self {
        let thread_id = unsafe { GetCurrentThreadId() };
        let hinstance = unsafe { GetModuleHandleW(None).unwrap() };
        let async_calls: Arc<Mutex<Vec<Job>>> = Default::default();
        let posted_async: Arc<std::sync::atomic::AtomicBool> = Default::default();

        // A message-only window (parent = HWND_MESSAGE) replaces the ATL window.
        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("RobloxPlayerWindow"),
                PCWSTR::null(),
                WS_POPUP,
                0,
                0,
                0,
                0,
                HWND_MESSAGE,
                None,
                hinstance,
                None,
            )
            .expect("FunctionMarshaller window")
        }
        .0 as usize;

        Self {
            hwnd,
            thread_id,
            async_calls,
            posted_async,
            ref_count: 0,
        }
    }

    /// `Execute` — run `job` on the marshaller's thread, blocking the caller.
    pub fn execute(&self, job: Job) {
        if self.thread_id == unsafe { GetCurrentThreadId() } {
            job();
        } else {
            let boxed = Box::into_raw(Box::new(Some(job)));
            unsafe {
                SendMessageW(
                    HWND(self.hwnd as _),
                    WM_EVENT,
                    WPARAM(0),
                    LPARAM(boxed as isize),
                );
            }
        }
    }

    /// `Submit` — enqueue `job` for async execution on the marshaller's thread.
    pub fn submit(&self, job: Job) {
        self.async_calls.lock().unwrap().push(job);
        if !self
            .posted_async
            .swap(true, std::sync::atomic::Ordering::SeqCst)
        {
            unsafe {
                let _ = PostMessageW(HWND(self.hwnd as _), WM_ASYNCEVENT, WPARAM(0), LPARAM(0));
            }
        }
    }

    /// `ProcessMessages` — drain only the async queue (call from owning thread).
    pub fn process_messages(&self) {
        let mut msg = MSG::default();
        unsafe {
            while PeekMessageW(
                &mut msg,
                HWND(self.hwnd as _),
                WM_ASYNCEVENT,
                WM_ASYNCEVENT,
                PM_REMOVE,
            )
            .as_bool()
            {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }

    fn drain_async(&self) {
        self.posted_async
            .store(false, std::sync::atomic::Ordering::SeqCst);
        let jobs: Vec<Job> = std::mem::take(&mut *self.async_calls.lock().unwrap());
        for job in jobs {
            job();
        }
    }
}

/// Window procedure equivalent of `OnEvent` / `OnAsyncEvent`.
pub unsafe extern "system" fn marshaller_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    this: *const FunctionMarshaller,
) -> LRESULT {
    match msg {
        WM_EVENT => {
            let boxed = lparam.0 as *mut Option<Job>;
            if let Some(job) = (unsafe { *boxed }).take() {
                job();
            }
            LRESULT(0)
        }
        WM_ASYNCEVENT => LRESULT(0),
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}
