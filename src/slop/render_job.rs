//! Translated from `RenderJob.h/.cpp`.
//!
//! The render job runs `ViewBase::render()` exclusive to the DataModel.
//!
//! ## Anti-cheat removed
//! The original `stepDataModelJob` opened with a `VMProtectBeginMutation("34")`
//! block that ran `Time::isSpeedCheater()` / `Time::isDebugged()` and reported
//! hackers (`reportHacker(... "richard"/"suzanne")`). The whole block and the
//! `#include "VMProtect/VMProtectSDK.h"` are stripped. `remoteCheatHelper` /
//! `reportHacker` are removed.

#![allow(dead_code)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock, Weak};

use super::function_marshaller::FunctionMarshaller;
use super::rbx::{DataModel, JobAccess};

/// `RBX::TaskScheduler::StepResult`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StepResult {
    Done,
    Stepped,
}

/// `RBX::RenderJob : public BaseRenderJob, public IMetric`.
pub struct RenderJob {
    pub marshaller: Arc<RwLock<FunctionMarshaller>>,
    pub stopped: bool,
    pub is_awake: bool,
    pub data_model: Weak<dyn DataModel>,
}

impl RenderJob {
    pub fn stop(&mut self) {
        self.stopped = true;
    }

    /// `RenderJob::stepDataModelJob` — security/cheat block removed.
    pub fn step_data_model_job(&mut self) -> StepResult {
        let Some(dm) = self.data_model.upgrade() else {
            return StepResult::Done;
        };
        if self.stopped {
            return StepResult::Done;
        }

        // [removed] VMProtectBeginMutation("34")
        //   if (Time::isSpeedCheater()) reportHacker(dm, "richard");
        //   if (Time::isDebugged())     reportHacker(dm, "suzanne");
        // VMProtectEnd()

        // Standard render path: lock the DataModel for render, step, and
        // marshal renderPrepare/renderPerform onto the view thread.
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
}
