//! Translated from `GameVerbs.h/.cpp`.
//!
//! The four UI verbs: leave game, screenshot (+ optional upload), record-toggle
//! (video capture + upload), and toggle-fullscreen. No anti-cheat code here.
//! Video capture (`DSVideoCaptureEngine`, `VideoControl`) and the engine's
//! `Verb`/`DataModel` machinery are external.

#![allow(dead_code)]

use std::sync::Arc;

use crate::rbx::{self, DataModel, Game, JobAccess};

/// Common `RBX::Verb` interface (engine base class).
pub trait Verb {
    fn do_it(&mut self);
    fn is_enabled(&self) -> bool {
        true
    }
    fn is_checked(&self) -> bool {
        false
    }
}

/// `LeaveGameVerb` — closes the window, ending the process.
pub struct LeaveGameVerb {
    view: *mut crate::view::View,
}
impl LeaveGameVerb {
    pub fn new(view: *mut crate::view::View) -> Self {
        Self { view }
    }
}
impl Verb for LeaveGameVerb {
    fn do_it(&mut self) {
        // MainLogManager::setLeaveGame()
        if !self.view.is_null() {
            unsafe { (*self.view).close_window() };
        }
    }
}

/// `ScreenshotVerb` — request a screenshot; on completion optionally upload.
pub struct ScreenshotVerb {
    game: Arc<dyn Game>,
}
impl ScreenshotVerb {
    pub fn new(game: Arc<dyn Game>) -> Self {
        // dataModel->screenshotReadySignal.connect(screenshotFinished) (engine)
        Self { game }
    }
}
impl Verb for ScreenshotVerb {
    fn do_it(&mut self) {
        if let Some(dm) = self.game.get_data_model() {
            dm.submit_task(Box::new(|| { /* DataModel::TakeScreenshotTask */ }), JobAccess::Write);
        }
    }
}

/// `RecordToggleVerb` — toggles video recording on a worker thread.
pub struct RecordToggleVerb {
    game: Arc<dyn Game>,
    recording: bool,
}
impl RecordToggleVerb {
    pub fn new(game: Arc<dyn Game>) -> Self {
        Self { game, recording: false }
    }
    pub fn start_action(&mut self) {
        // videoControl->startRecording(soundService); videoRecordingSignal(true)
        self.recording = true;
    }
    pub fn stop_action(&mut self) {
        // videoControl->stopRecording(); maybe uploadVideo()
        self.recording = false;
    }
}
impl Verb for RecordToggleVerb {
    fn do_it(&mut self) {
        if self.recording {
            self.stop_action();
        } else {
            self.start_action();
        }
    }
    fn is_enabled(&self) -> bool {
        // GameSettings::videoCaptureEnabled && isUploadingVideo()
        true
    }
    fn is_checked(&self) -> bool {
        self.recording
    }
}

/// `ToggleFullscreenVerb`.
pub struct ToggleFullscreenVerb {
    view: *mut crate::view::View,
}
impl ToggleFullscreenVerb {
    pub fn new(view: *mut crate::view::View) -> Self {
        Self { view }
    }
}
impl Verb for ToggleFullscreenVerb {
    fn do_it(&mut self) {
        if !self.view.is_null() {
            unsafe {
                let v = &mut *self.view;
                v.set_fullscreen(!v.is_fullscreen());
            }
        }
    }
    fn is_checked(&self) -> bool {
        !self.view.is_null() && unsafe { (*self.view).is_fullscreen() }
    }
}

/// Helper used by screenshot/video upload paths (`PostImageFinished`).
pub(crate) fn post_image_finished(response: &str, ok: bool, dm: &Arc<dyn DataModel>) {
    let _ = (response, ok, dm);
    let _ = rbx::base_url();
}
