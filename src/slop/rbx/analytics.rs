//! Boundary for `RBX::Analytics`, `RobloxGoogleAnalytics`, `Stats`, logging.
//!
//! EXTERNAL: `RBX::Analytics::InfluxDb::Points`,
//! `RBX::Analytics::EphemeralCounter`, `RBX::RobloxGoogleAnalytics`,
//! `RBX::Stats`, `RBX::StandardOut`, `LogManager`.

#![allow(dead_code, unused_variables)]

/// `RBX::Analytics::InfluxDb::Points`.
#[derive(Default)]
pub struct Points;
impl Points {
    pub fn add_point(&mut self, key: &str, value: impl ToString) {}
    pub fn report(&self, name: &str, hundredths_percent: i32) {}
}

/// `RBX::Analytics::EphemeralCounter`.
pub fn report_counter(name: &str, value: i32, important: bool) {}
pub fn report_stats(name: &str, value: f64) {}

/// `RBX::Analytics::setReporter/setLocation/setAppVersion`.
pub fn set_reporter(name: &str) {}
pub fn set_location(loc: &str) {}
pub fn set_app_version(ver: &str) {}

/// `RBX::RobloxGoogleAnalytics`.
pub fn track_user_timing(category: &str, action: &str, ms: f64, label: &str) {}
pub fn track_event(category: &str, action: &str, label: &str) {}

/// `LogManager::ReportEvent` levels (`EVENTLOG_*`).
#[derive(Clone, Copy)]
pub enum EventType {
    Information,
    Warning,
    Error,
}

pub fn report_event(kind: EventType, message: &str) {}

/// `RBX::StandardOut::singleton()->printf(...)`.
pub fn standard_out(level: &str, message: &str) {
    eprintln!("{level}: {message}");
}
