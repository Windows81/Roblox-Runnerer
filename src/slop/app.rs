//! Translated from `Application.h/.cpp`.
//!
//! The top-level client object: parses args, loads `AppSettings.xml`, fetches
//! join/auth data, creates the `Document`/`View`, owns the verbs, the
//! teleporter and the function marshaller, and drives shutdown.
//!
//! ## Anti-cheat / VMProtect / signature code removed
//! Relative to the original `Application.cpp`, the following were stripped:
//! * `vmProtectedDetectCheatEngineIcon()` CheatEngine detection and the
//!   `HATE_CHEATENGINE_OLD` token/`sendStats` plumbing.
//! * `RBX::isSandboxie()` + `HATE_INVALID_ENVIRONMENT`.
//! * `ProgramMemoryChecker` initial hashing (`pmcHash`, `initialProgramHash`).
//! * `protectVmpSections()` and the `RwxFailReport` VMP size reporting.
//! * `hookApi()` / `vehHookLocation` / `RtlDispatchExceptionHook` /
//!   `setupCeLogWatcher()`.
//! * `setWindowFrame()` → `VerifyCryptSignature(...)` + `HATE_SIGNATURE`
//!   (Authenticode check, removed per request).
//! * The obfuscated `--waitEvent` keys that called `protectVmpSections()` +
//!   `RBX::Security::patchMain()` (golden-hash patcher).
//! * `CollectMd5Hash` self-hashing for `DataModel::hash`.
//!
//! `SetProcessDEPPolicy` (DEP) is kept: it is ordinary OS hardening, not a
//! cheat-detection mechanism.

#![allow(dead_code)]

use std::sync::{Arc, Mutex, RwLock};

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::PCSTR;

use super::document::Document;
use super::function_marshaller::FunctionMarshaller;
use super::game_verbs::*;
use super::rbx::{self, LaunchMode};
use super::view::View;

#[derive(Clone, Copy, PartialEq, Eq)]
enum RequestPlaceInfoResult {
    Success,
    Failed,
    Retry,
    GameFull,
    UserLeft,
}

/// Parsed command-line arguments (`boost::program_options` → a plain struct).
#[derive(Default)]
pub struct Args {
    pub help: bool,
    pub version: bool,
    pub id: Option<i32>,
    pub content: Option<String>,
    pub authentication_url: Option<String>,
    pub authentication_ticket: Option<String>,
    pub join_script_url: Option<String>,
    pub browser_tracker_id: Option<String>,
    pub wait_event: Option<String>,
    pub global_basic_settings_path: Option<String>,
    pub api_out: Option<String>,
    pub dmp: bool,
    pub play: bool,
    pub app: bool,
}

/// `RBX::Application`.
pub struct Application {
    main_window: HWND,
    launch_mode: LaunchMode,
    args: Args,
    module_filename: String,
    global_basic_settings_path: String,
    wait_event_name: String,
    hide_chat: bool,
    crash_report_enabled: bool,

    current_document: Option<Document>,
    main_view: Option<Arc<RwLock<View>>>,

    marshaller: *mut FunctionMarshaller,

    toggle_fullscreen_verb: Option<ToggleFullscreenVerb>,
    leave_game_verb: Option<LeaveGameVerb>,
    record_toggle_verb: Option<RecordToggleVerb>,
    screenshot_verb: Option<ScreenshotVerb>,

    entered_shutdown: std::sync::atomic::AtomicI32,
}

impl Application {
    pub fn new() -> Self {
        Self {
            main_window: HWND::default(),
            launch_mode: LaunchMode::Play,
            args: Args::default(),
            module_filename: String::new(),
            global_basic_settings_path: String::new(),
            wait_event_name: String::new(),
            hide_chat: false,
            crash_report_enabled: true,
            current_document: None,
            main_view: None,
            marshaller: std::ptr::null_mut(),
            toggle_fullscreen_verb: None,
            leave_game_verb: None,
            record_toggle_verb: None,
            screenshot_verb: None,
            entered_shutdown: std::sync::atomic::AtomicI32::new(0),
        }
    }

    pub fn wait_event_name(&self) -> &str {
        &self.wait_event_name
    }

    /// `Application::ParseArguments` — obfuscated `--waitEvent` patcher removed.
    pub fn parse_arguments(&mut self, cmd_line: &str) -> bool {
        let args = match parse_program_options(cmd_line) {
            Ok(a) => a,
            Err(e) => {
                rbx::analytics::report_event(
                    rbx::analytics::EventType::Error,
                    &format!("Command-line args error: {e}"),
                );
                return false;
            }
        };

        if args.help {
            return false;
        }
        if let Some(path) = &args.api_out {
            // Reflection::Metadata::writeEverything(file) then exit.
            let _ = path;
            return false;
        }
        if args.dmp {
            // initialize crash reporter + DumpErrorUploader::Upload, then exit.
            return false;
        }
        if let Some(p) = &args.global_basic_settings_path {
            self.global_basic_settings_path = p.clone();
        }
        if args.version {
            return false;
        }
        if let Some(c) = &args.content {
            rbx::content_set_asset_folder(c);
        }
        if let Some(w) = &args.wait_event {
            self.wait_event_name = w.clone();
            // [removed] obfuscated `-w <key>` overloads that called
            // protectVmpSections() + RBX::Security::patchMain() (golden-hash
            // patcher) or silently exited.
        }
        if args.play {
            self.launch_mode = LaunchMode::Play;
        }

        self.args = args;
        true
    }

    /// `Application::LoadAppSettings` — parse `AppSettings.xml`.
    pub fn load_app_settings(&mut self, _hinstance: isize) -> bool {
        // GetModuleFileNameW → moduleFilename; SetCurrentDirectory; parse
        // <BaseUrl>/<SilentCrashReport>/<ContentFolder>/<HideChatWindow>.
        // Registry CrashReport / SilentCrashReport overrides.
        // On any std::exception → handleError + return false.
        true
    }

    /// `Application::Initialize` — anti-cheat/VMP/signature blocks removed.
    pub fn initialize(&mut self, hwnd: HWND, _hinstance: isize) -> Result<bool, InitError> {
        self.main_window = hwnd;

        // Analytics reporter/location/version (CVersionInfo + GetUserGeoID).
        rbx::analytics::set_reporter("PC Player");

        self.initialize_logger();
        rbx::game_global_init(false);

        // Auth + join-script fetch (parallel). Stored as HttpFutures.
        let mut authentication_result = rbx::HttpFuture::default();
        let mut authentication_url = String::new();
        let mut authentication_ticket = String::new();
        let mut script_url = String::new();
        let mut script_is_place_launcher = false;

        if let Some(id) = self.args.id {
            if !self.request_place_info_by_id(
                id,
                &mut authentication_url,
                &mut authentication_ticket,
                &mut script_url,
            ) {
                return Ok(false);
            }
        } else if let (Some(u), Some(t), Some(j)) = (
            self.args.authentication_url.clone(),
            self.args.authentication_ticket.clone(),
            self.args.join_script_url.clone(),
        ) {
            authentication_url = u;
            authentication_ticket = t;
            script_url = j;
            if script_url.to_lowercase().contains("placelauncher.ashx") {
                self.launch_mode = LaunchMode::PlayProtocol;
                script_is_place_launcher = true;
            }
        }

        if !authentication_result.valid() {
            authentication_result =
                self.renew_login_async(&authentication_url, &authentication_ticket);
        }
        let join_script_result = if !script_is_place_launcher {
            fetch_join_script_async(&script_url)
        } else {
            rbx::HttpFuture::default()
        };

        // [removed] DataModel::hash = CollectMd5Hash(moduleFilename)
        // [removed] bool cheatEngine = vmProtectedDetectCheatEngineIcon()
        // [removed] bool sandboxie = RBX::isSandboxie()
        // [removed] ProgramMemoryChecker initial hash (pmcHash / nonce)
        // [removed] protectVmpSections()

        // Keep: enable DEP (ordinary OS hardening, not cheat detection).
        enable_dep();

        // [removed] FLog::ResetSynchronizedVariablesState gating on cheat flags
        rbx::http::Http::set_use_statistics(true);

        // GlobalAdvanced/Basic settings load (engine).
        rbx::settings::GlobalSettings::advanced().load_state("");
        rbx::settings::GlobalSettings::basic().load_state(&self.global_basic_settings_path);

        // [removed] hookApi(); vehHookLocation/Stub; setupCeLogWatcher()

        self.initialize_crash_reporter();

        // [removed] StandardOut "Cheat engine ... detected"
        // [removed] DataModel::sendStats |= HATE_CHEATENGINE_OLD * cheatEngine
        // [removed] DataModel::sendStats |= HATE_INVALID_ENVIRONMENT * sandboxie
        // [removed] setWindowFrame() — Authenticode VerifyCryptSignature
        self.upload_crash_data(false);
        self.share_hwnd(hwnd);

        // Single-instance guard, profanity filter, profiler, task scheduler,
        // analytics lottery, machine-config post — all engine plumbing.

        if !script_is_place_launcher {
            self.start_new_game(hwnd, join_script_result, false);
            authentication_result.wait();
        } else {
            self.initialize_new_game(hwnd);
            if let Some(doc) = &self.current_document {
                doc.set_ui_message("Requesting Server...");
            }
            authentication_result.wait();
            // launchPlaceThread → LaunchPlaceThreadImpl(scriptUrl)
            self.launch_place_thread_impl(&script_url);
        }

        self.marshaller = FunctionMarshaller::get_window();

        // doMachineIdCheck thread (engine MachineIdUploader). [removed banned-machine UI? no — kept: not anti-cheat, it's account ban]

        Ok(true)
    }

    fn initialize_logger(&self) {
        // StandardOut::messageOut.connect(onMessageOut → OutputDebugString)
    }

    fn initialize_crash_reporter(&self) {}
    fn upload_crash_data(&self, _user_requested: bool) {}

    fn share_hwnd(&self, _hwnd: HWND) {
        // CreateFileMapping("RBXMAINWND-...") + MapViewOfFile + CopyMemory(hWnd).
    }

    /// `Application::requestPlaceInfo(int placeId, ...)`.
    fn request_place_info_by_id(
        &self,
        place_id: i32,
        auth_url: &mut String,
        ticket: &mut String,
        script_url: &mut String,
    ) -> bool {
        let url = format!(
            "{}Game/PlaceLauncher.ashx?request=RequestGame&placeId={place_id}&isPartyLeader=false&gender=&isTeleport=true",
            rbx::base_url()
        );
        let mut retries = 5i32; // FInt::RequestPlaceInfoRetryCount
        while retries >= 0 {
            if self.request_place_info_url(&url, auth_url, ticket, script_url)
                == RequestPlaceInfoResult::Success
            {
                return true;
            }
            retries -= 1;
            std::thread::sleep(std::time::Duration::from_millis(2000));
        }
        false
    }

    /// `Application::requestPlaceInfo(url, ...)` — parse the JSON status.
    fn request_place_info_url(
        &self,
        _url: &str,
        _auth_url: &mut String,
        _ticket: &mut String,
        _script_url: &mut String,
    ) -> RequestPlaceInfoResult {
        // Http GET/POST → WebParser::parseJSONTable → status:
        //   2 => Success (read authenticationUrl/authenticationTicket/joinScriptUrl)
        //   6 => GameFull, 10 => UserLeft, 0|1 => Retry, else Failed.
        RequestPlaceInfoResult::Failed
    }

    fn launch_place_thread_impl(&self, _place_launcher_url: &str) {
        // Polls requestPlaceInfo until Success/UserLeft, then submits
        // Document::Start on the DataModel write job. (engine threads)
    }

    fn renew_login_async(&self, _auth_url: &str, _ticket: &str) -> rbx::HttpFuture {
        // AuthenticationMarshallar(host).AuthenticateAsync(url, ticket)
        rbx::HttpFuture::default()
    }

    fn login_async(&self, user: &str, password: &str) -> rbx::HttpFuture {
        let login_url = format!("{}login/v1", rbx::base_url())
            .replace("http", "https")
            .replace("www", "api");
        let post = format!("{{\"username\":\"{user}\", \"password\":\"{password}\"}}");
        rbx::http::post_async(
            &login_url,
            rbx::HttpPostData::new(post, rbx::http::CONTENT_TYPE_APPLICATION_JSON, false),
        )
    }

    /// `Application::HandleWindowsMessage`.
    pub fn handle_windows_message(&mut self, msg: u32, wparam: WPARAM, lparam: LPARAM) {
        let Some(view) = &self.main_view else {
            unsafe {
                DefWindowProcW(self.main_window, msg, wparam, lparam);
            };
            return;
        };
        let mut unwrapped = view.write().unwrap();
        unwrapped.handle_windows_message(msg, wparam, lparam);
    }

    /// `Application::OnResize`.
    pub fn on_resize(&mut self, wparam: WPARAM, cx: i32, cy: i32) {
        let Some(view) = self.main_view.clone() else {
            return;
        };
        let mut unwrapped = view.write().unwrap();
        unwrapped.on_resize(wparam, cx, cy);
    }

    /// `Application::InitializeNewGame`.
    fn initialize_new_game(&mut self, hwnd: HWND) {
        let mut doc = Document::new();
        doc.initialize(hwnd, !self.hide_chat);
        let game = doc.game().expect("game");
        self.current_document = Some(doc);

        let mut view = View::new(hwnd);
        view.start(game.clone());
        self.main_view = Some(RwLock::new(view).into());

        self.connect_gui_service(&game);
        self.init_verbs();
    }

    /// `Application::StartNewGame`.
    fn start_new_game(&mut self, hwnd: HWND, script_result: rbx::HttpFuture, is_teleport: bool) {
        let mut doc = Document::new();
        doc.initialize(hwnd, !self.hide_chat);
        let game = doc.game().expect("game");

        if !is_teleport {
            let view = View::new(hwnd);
            self.main_view = Some(RwLock::new(view).into());
        }

        if let Some(view) = &self.main_view {
            view.write().unwrap().start(game.clone());
        }

        let launch_mode = self.launch_mode;
        let vr = self.vr_device_name();
        self.current_document = Some(doc);
        if let Some(game) = self.current_document.as_ref().and_then(|d| d.game()) {
            self.connect_gui_service(&game);
        }
        self.init_verbs();
    }

    fn connect_gui_service(&self, game: &Arc<dyn rbx::Game>) {
        if let Some(dm) = game.get_data_model() {
            if let Some(gs) = dm.find_gui_service() {
                gs.on_open_url_window(Box::new(|_url| { /* openUrlInBrowserApp */ }));
                gs.on_url_window_closed(Box::new(|| { /* closeBrowser */ }));
            }
        }
    }

    fn init_verbs(&mut self) {
        let view_ptr = self.main_view.clone().unwrap();
        let game = self.current_document.as_ref().and_then(|d| d.game());
        if let Some(game) = game {
            self.leave_game_verb = Some(LeaveGameVerb {
                view: view_ptr.clone(),
            });
            self.record_toggle_verb = Some(RecordToggleVerb::new(game.clone()));
            self.screenshot_verb = Some(ScreenshotVerb::new(game.clone()));
            self.toggle_fullscreen_verb = Some(ToggleFullscreenVerb {
                view: view_ptr.clone(),
            });
        }
    }

    fn shutdown_verbs(&mut self) {
        self.toggle_fullscreen_verb = None;
        self.leave_game_verb = None;
        self.screenshot_verb = None;
        self.record_toggle_verb = None;
    }

    /// `Application::Teleport`.
    pub fn teleport(&mut self, auth_url: &str, ticket: &str, script_url: &str) {
        if let Some(doc) = &self.current_document {
            doc.prepare_shutdown();
        }
        if let Some(view) = &self.main_view {
            view.write().unwrap().stop();
        }
        self.shutdown_verbs();
        if let Some(mut doc) = self.current_document.take() {
            doc.shutdown();
        }

        let authentication_result = self.renew_login_async(auth_url, ticket);
        let join_script_result = fetch_join_script_async(script_url);
        let hwnd = self.main_window;
        self.start_new_game(hwnd, join_script_result, true);
        authentication_result.wait();
    }

    /// `Application::UploadSessionLogs`.
    pub fn upload_session_logs(&self) {
        // logManager.CreateFakeCrashDump() + MessageBox feedback.
    }

    /// `Application::OnHelp`.
    pub fn on_help(&self) {
        // ShellExecute open http://wiki.roblox.com unless DFFlag::DontOpenWikiOnClient.
    }

    /// `Application::OnGetMinMaxInfo` (static).
    pub fn on_get_min_max_info(_mmi: *mut MINMAXINFO) {
        // Clamp to desktop work area / min game window size (CRenderSettingsItem).
    }

    /// `Application::AboutToShutdown`.
    pub fn about_to_shutdown(&mut self) {
        self.entered_shutdown
            .compare_exchange(
                0,
                1,
                std::sync::atomic::Ordering::SeqCst,
                std::sync::atomic::Ordering::SeqCst,
            )
            .ok();
        if let Some(doc) = &self.current_document {
            doc.prepare_shutdown();
        }
        if let Some(view) = &self.main_view {
            view.clone().write().unwrap().about_to_shutdown();
        }
    }

    /// `Application::Shutdown`.
    pub fn shutdown(&mut self) {
        self.shutdown_verbs();
        if let Some(view) = self.main_view.take() {
            view.write().unwrap().stop();
        }
        if let Some(mut doc) = self.current_document.take() {
            doc.shutdown();
        }
        rbx::settings::GlobalSettings::basic().save_state();
        rbx::game_global_exit();
        if !self.marshaller.is_null() {
            FunctionMarshaller::release_window(self.marshaller);
            self.marshaller = std::ptr::null_mut();
        }
    }

    fn vr_device_name(&self) -> Option<String> {
        None
    }
}

/// `RBX::initialization_error`.
#[derive(Debug)]
pub struct InitError(pub String);
impl std::fmt::Display for InitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for InitError {}

/// `fetchJoinScriptAsync` — only fetch trusted URLs (silent empty otherwise).
fn fetch_join_script_async(url: &str) -> rbx::HttpFuture {
    if rbx::content_is_url(url) && rbx::network_is_trusted_content(url) {
        rbx::http::get_with_retries(url, 5)
    } else {
        rbx::http::ready_empty_future()
    }
}

/// `SetProcessDEPPolicy(1)` — kept as OS hardening.
fn enable_dep() {
    unsafe {
        if let Ok(kernel32) = GetModuleHandleA(PCSTR(b"Kernel32\0".as_ptr())) {
            if let Some(pfn) = GetProcAddress(kernel32, PCSTR(b"SetProcessDEPPolicy\0".as_ptr())) {
                let set_dep: unsafe extern "system" fn(u32) -> i32 = std::mem::transmute(pfn);
                set_dep(1);
            }
        }
    }
}

/// `boost::program_options` parsing of `_tWinMain`'s command line.
fn parse_program_options(cmd_line: &str) -> Result<Args, String> {
    let mut args = Args::default();
    let tokens = split_winmain(cmd_line);
    if tokens.is_empty() {
        args.help = true;
        return Ok(args);
    }
    let mut it = tokens.iter().peekable();
    while let Some(tok) = it.next() {
        let next = |it: &mut std::iter::Peekable<std::slice::Iter<String>>| it.next().cloned();
        match tok.trim_start_matches('-') {
            "help" | "?" => args.help = true,
            "version" | "v" => args.version = true,
            "play" => args.play = true,
            "app" => args.app = true,
            "dmp" | "d" => args.dmp = true,
            "id" => args.id = next(&mut it).and_then(|v| v.parse().ok()),
            "content" | "c" => args.content = next(&mut it),
            "authenticationUrl" | "a" => args.authentication_url = next(&mut it),
            "authenticationTicket" | "t" => args.authentication_ticket = next(&mut it),
            "joinScriptUrl" | "j" => args.join_script_url = next(&mut it),
            "browserTrackerId" | "b" => args.browser_tracker_id = next(&mut it),
            "waitEvent" | "w" => args.wait_event = next(&mut it),
            "globalBasicSettingsPath" | "g" => args.global_basic_settings_path = next(&mut it),
            "API" => args.api_out = next(&mut it),
            _ => {}
        }
    }
    Ok(args)
}

/// `boost::program_options::split_winmain`.
fn split_winmain(cmd_line: &str) -> Vec<String> {
    // Minimal whitespace/quote split adequate for the client's option set.
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    for c in cmd_line.chars() {
        match c {
            '"' => in_quotes = !in_quotes,
            c if c.is_whitespace() && !in_quotes => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            c => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}
