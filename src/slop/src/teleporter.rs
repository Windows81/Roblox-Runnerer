//! Translated from `Teleporter.h/.cpp`.
//!
//! Implements `RBX::TeleportCallback`: when the engine requests a teleport it
//! marshals `Application::Teleport` onto the main thread.

#![allow(dead_code)]

use crate::function_marshaller::FunctionMarshaller;
use crate::rbx;

/// `RBX::Teleporter : public TeleportCallback`.
pub struct Teleporter {
    app: *mut crate::app::Application,
    marshaller: *mut FunctionMarshaller,
}

impl Teleporter {
    pub fn new() -> Self {
        Self { app: std::ptr::null_mut(), marshaller: std::ptr::null_mut() }
    }

    pub fn initialize(
        &mut self,
        app: *mut crate::app::Application,
        marshaller: *mut FunctionMarshaller,
    ) {
        self.app = app;
        self.marshaller = marshaller;

        // EXTERNAL: TeleportService::SetCallback / SetBaseUrl (engine).
        rbx_teleport_service_set_base_url(&rbx::base_url());
    }

    /// `doTeleport` — engine callback. Marshals `Application::Teleport`.
    pub fn do_teleport(&self, url: String, ticket: String, script: String) {
        let app = self.app;
        if !self.marshaller.is_null() {
            unsafe {
                (*self.marshaller).submit(Box::new(move || {
                    if !app.is_null() {
                        unsafe { (*app).teleport(&url, &ticket, &script) };
                    }
                }));
            }
        }
    }

    pub fn is_teleport_enabled(&self) -> bool {
        true
    }
}

// EXTERNAL: RBX::TeleportService — defined in v8datamodel.
fn rbx_teleport_service_set_base_url(_url: &str) {}
