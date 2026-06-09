//! Translated from `FunctionMarshaller.h/.cpp`.
//!
//! A helper for marshalling closures onto a specific Windows thread, both
//! synchronously (`Execute`, via `SendMessage`) and asynchronously (`Submit`,
//! via a queue drained on `WM_ASYNCEVENT`). The original is an
//! `ATL::CWindowImpl` keyed per-thread in a static map; this is an idiomatic
//! Rust equivalent built on a hidden message-only `HWND`.

#![allow(dead_code)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::*;

const WM_EVENT: u32 = WM_USER + 101;
const WM_ASYNCEVENT: u32 = WM_USER + 102;

type Job = Box<dyn FnOnce() + Send>;

/// One marshaller per thread, mirroring the C++ static `windows` map.
pub struct FunctionMarshaller {
    hwnd: HWND,
    thread_id: u32,
    async_calls: Arc<Mutex<Vec<Job>>>,
    posted_async: Arc<std::sync::atomic::AtomicBool>,
    ref_count: usize,
}

thread_local! {
    static THREAD_MARSHALLERS: RefCell<HashMap<u32, *mut FunctionMarshaller>> =
        RefCell::new(HashMap::new());
}

impl FunctionMarshaller {
    /// `FunctionMarshaller::GetWindow()` — share one instance per thread.
    pub fn get_window() -> *mut FunctionMarshaller {
        let tid = unsafe { GetCurrentThreadId() };
        THREAD_MARSHALLERS.with(|m| {
            let mut map = m.borrow_mut();
            if let Some(&existing) = map.get(&tid) {
                unsafe { (*existing).ref_count += 1 };
                existing
            } else {
                let boxed = Box::into_raw(Box::new(Self::new(tid)));
                map.insert(tid, boxed);
                unsafe { (*boxed).ref_count += 1 };
                boxed
            }
        })
    }

    /// `FunctionMarshaller::ReleaseWindow()`.
    pub fn release_window(window: *mut FunctionMarshaller) {
        let tid = unsafe { (*window).thread_id };
        THREAD_MARSHALLERS.with(|m| {
            let mut map = m.borrow_mut();
            unsafe { (*window).ref_count -= 1 };
            if unsafe { (*window).ref_count } == 0 {
                map.remove(&tid);
                unsafe {
                    let _ = DestroyWindow((*window).hwnd);
                    drop(Box::from_raw(window));
                }
            }
        });
    }

    fn new(thread_id: u32) -> Self {
        let hinstance = unsafe { GetModuleHandleW(None).unwrap() };
        let async_calls: Arc<Mutex<Vec<Job>>> = Default::default();
        let posted_async: Arc<std::sync::atomic::AtomicBool> = Default::default();

        // A message-only window (parent = HWND_MESSAGE) replaces the ATL window.
        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("Roblox.FunctionMarshaller"),
                PCWSTR::null(),
                WS_POPUP,
                0,
                0,
                0,
                0,
                Some(HWND_MESSAGE),
                None,
                Some(hinstance.into()),
                None,
            )
            .expect("FunctionMarshaller window")
        };

        Self { hwnd, thread_id, async_calls, posted_async, ref_count: 0 }
    }

    /// `Execute` — run `job` on the marshaller's thread, blocking the caller.
    pub fn execute(&self, job: Job) {
        if self.thread_id == unsafe { GetCurrentThreadId() } {
            job();
        } else {
            let boxed = Box::into_raw(Box::new(Some(job)));
            unsafe {
                SendMessageW(
                    self.hwnd,
                    WM_EVENT,
                    Some(WPARAM(0)),
                    Some(LPARAM(boxed as isize)),
                );
            }
        }
    }

    /// `Submit` — enqueue `job` for async execution on the marshaller's thread.
    pub fn submit(&self, job: Job) {
        self.async_calls.lock().unwrap().push(job);
        if !self.posted_async.swap(true, std::sync::atomic::Ordering::SeqCst) {
            unsafe {
                let _ = PostMessageW(Some(self.hwnd), WM_ASYNCEVENT, WPARAM(0), LPARAM(0));
            }
        }
    }

    /// `ProcessMessages` — drain only the async queue (call from owning thread).
    pub fn process_messages(&self) {
        let mut msg = MSG::default();
        unsafe {
            while PeekMessageW(&mut msg, Some(self.hwnd), WM_ASYNCEVENT, WM_ASYNCEVENT, PM_REMOVE)
                .as_bool()
            {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }

    fn drain_async(&self) {
        self.posted_async.store(false, std::sync::atomic::Ordering::SeqCst);
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
            if let Some(job) = (*boxed).take() {
                job();
            }
            drop(Box::from_raw(boxed));
            LRESULT(0)
        }
        WM_ASYNCEVENT => {
            if !this.is_null() {
                (*this).drain_async();
            }
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
