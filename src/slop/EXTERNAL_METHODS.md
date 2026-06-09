# External methods — symbols called but **not** defined in `WindowsClient`

This satisfies requirement (3). Every entry below is *called* by the original
`WindowsClient` C++ but *defined elsewhere* — in the wider ROBLOX engine, in a
third-party library (Boost, G3D, ATL/MFC), or in the Win32/CRT/COM platform
SDK. The Rust port routes all of the ROBLOX-engine ones through
`src/rbx/` (see that module).

Legend: 🔴 = used **only** by code that was removed (anti-cheat / VMProtect /
Authenticode), so it has no counterpart in the Rust port.

---

## 1. ROBLOX engine — `RBX::` and related (the bulk)

### DataModel / Game / services
- `RBX::SecurePlayerGame` (ctor)
- `RBX::Game` — `getDataModel`, `shutdown`, `configurePlayer`, `globalInit`, `globalExit`, `gameLoadedSignal`
- `RBX::DataModel` — `submitTask`, `isClosed`, `setUiMessage`, `clearUiMessage`, `find<T>`, `create<T>`, `getWorkspace`, `getDataModel`, `get`, `getScreenshotSEOInfo`, `getPlaceID`, `getVideoSEOInfo`, `isVideoSEOInfoSet`, `mouseStats`, `screenshotReadySignal`, `gameLoadedSignal`, `addHackFlag` 🔴, `sendStats` 🔴, `hash` 🔴, `renderStep`, `scoped_write_request`, `scoped_write_transfer`, `LegacyLock`, `ShowMessage`, `TakeScreenshotTask`, `ScreenshotUploadTask`
- `RBX::DataModelJob::{Write,Render}`
- `RBX::ServiceProvider::{create,find}<T>`
- `RBX::GuiService` — `setUiMessage`, `openUrlWindow`, `urlWindowClosed`, `UIMESSAGE_INFO`
- `RBX::CoreGuiService::displayOnScreenMessage`
- `RBX::UserInputService` — `setKeyboardEnabled`, `setMouseEnabled`, `setKeyState`, `getMouseWrapMode`, `setMouseWrapMode`, `getModifiedKey`, `fireInputEvent`, wrap-mode enum (`WRAP_*`)
- `RBX::ControllerService::setHardwareDevice`
- `RBX::Soundscape::SoundService::muteAllChannels`
- `RBX::Workspace` — `getCamera`, `onWrapMouse`
- `RBX::Camera` — `getCameraType`, `hasClientPlayer`, `getViewportWidth/Height`, `CUSTOM_CAMERA`
- `RBX::Network::Players::getLocalPlayer`, `RBX::Network::Player::reportStat`
- `RBX::Network::isTrustedContent`, `RBX::Network::getSystemUrlLocal` 🔴
- `RBX::ScriptContext` — `setTimeout`, `executeInNewThread`
- `RBX::ProtectedString::fromTrustedSource`, `RBX::ContentProvider::{isUrl,setAssetFolder,verifyScriptSignature}`
- `RBX::TeleportService::{SetCallback,SetBaseUrl,SetBrowserTrackerId}`, `RBX::TeleportCallback`
- `RBX::Instance` / `RBX::Creatable<Instance>::create<InputObject>`, `RBX::InputObject` (+ type/state enums, `setInputState/setPosition/setDelta`)
- `RBX::Verb`, `RBX::IDataState`, `RBX::VerbContainer`
- `RBX::ViewBase` — `CreateView`, `InitPluginModules`, `initResources`, `render*`, `onResize`, `bindWorkspace`, `buildGui`, `getFrameRateManager`, `updateVR`, `getVRDeviceName`, `getAndClearDoScreenshot`, `getMetricValue`
- `RBX::BaseRenderJob`, `RBX::FrameRateManager` (`GetFrameTimeStats`, `GetRenderTimeAverage`, `IsBlockCullingEnabled`, `getAntialiasingMode`)
- `RBX::IMetric`

### Settings / render settings
- `RBX::ClientAppSettings::singleton` (+ `GetValue*` getters)
- `RBX::GlobalBasicSettings::singleton`, `RBX::GlobalAdvancedSettings::singleton` (`loadState`, `saveState`, `removeInvalidChildren`)
- `RBX::GameBasicSettings::singleton` (`getFullScreen/setFullScreen`, `getStartMaximized/setStartMaximized`, `getStartScreenPos/Size`, `setStart*`, `getMouseSensitivity`, `inMousepanMode`, `getCanMousePan`, `getUploadVideoSetting`, `setUploadVideoSetting`)
- `RBX::GameSettings::singleton` (`getPostImageSetting/setPostImageSetting`, `videoCaptureEnabled`, `videoRecordingSignal`, upload/post enums)
- `RBX::DebugSettings::singleton` (`getErrorReporting`, `videoMemory`, `gfxcard`, report enums)
- `CRenderSettingsItem::singleton` (`getLatchedGraphicsMode`, `getWindowSize/setWindowSize`, `getFullscreenSize/setFullscreenSize`, `getResolutionPreference`, `getResolutionPreset`, `getMinFrameRate/getMaxFrameRate`, `minGameWindowSize`)
- `RBX::CRenderSettings::{GraphicsMode,ResolutionPreset,RESOLUTIONENTRY,...}`

### Util / analytics / logging / http
- `RBX::format`, `format_string`
- `RBX::StandardOut::singleton` (`print`, `printf`, `messageOut`)
- `LogManager` / `MainLogManager` — `ReportEvent`, `ReportException`, `getMainLogManager`, `NotifyFGThreadAlive`, `WriteCrashDump`, `getCrashEventName`, `hasCrashLogs`, `gatherCrashLogs`, `CreateFakeCrashDump`, `EnableImmediateCrashUpload`, `setGameLoaded`, `setLeaveGame`
- `RBX::Analytics::{setReporter,setLocation,setAppVersion}`, `RBX::Analytics::InfluxDb::Points`, `RBX::Analytics::EphemeralCounter::{reportCounter,reportStats}`, `RBX::Analytics::GoogleAnalytics::{lotteryInit,trackEvent}`
- `RBX::RobloxGoogleAnalytics::{init,setCanUseAnalytics,trackUserTiming,trackEvent,trackEventWithoutThrottling}`
- `RBX::Stats::{setBrowserTrackerId,reportGameStatus}`
- `RBX::Http` (`get`, `post`, `additionalHeaders`, `SetUseCurl`, `SetUseStatistics`, `trustCheckBrowser`, content-type consts), `RBX::HttpAsync::{getWithRetries,post}`, `RBX::HttpFuture`, `RBX::HttpPostData`
- `RBX::RegistryUtil::{read32bitNumber,write32bitNumber}`
- `RBX::FileSystem::getUserDirectory` (+ `DirExe/DirPicture/DirVideo`)
- `RBX::MachineIdUploader::{uploadMachineId,kBannedMachineMessage}`
- `RBX::postMachineConfiguration`, `RBX::ClientAppSettings`/client-settings free fns (`FetchClientSettingsData`, `LoadClientSettingsFromString`, `GetBaseURL`, `SetBaseURL`, `GetDmpUrl`)
- `RBX::Reflection::Metadata::writeEverything`, `RBX::Reflection::EnumDesc<T>`, `RBX::Reflection::ValueTable`, `WebParser::parseJSONTable`
- `RBX::Time::{now<Fast>,nowFast,nowFastSec,Interval}`, `RBX::ProfanityFilter::getInstance`, `RBX::Profiler::onThreadCreate`, `RBX::TaskScheduler::singleton` (`add`, `removeBlocking`, `setThreadCount`), `TaskSchedulerSettings::singleton`, `RBX::thread_wrapper`, `RBX::ScopedAssign`, `RBX::SystemUtil::{getGPUMake,osVer,getVideoMemory}`
- `RBX::Security::Impersonator` (+ `COM`, `RobloxGameScript_` identities)
- `CProcessPerfCounter`, `DumpErrorUploader` (`Upload`, `InitCrashEvent`), `RobloxCrashReporter::silent`, `CVersionInfo` (`Load`, `GetFileVersionAsDotString`), `AuthenticationMarshallar` (`Authenticate`, `AuthenticateAsync`)
- Fast-flag/var/log macros: `FFlag`, `DFFlag`, `FInt`, `DFInt`, `FString`, `FLog`, `DFLog`, `FASTLOG*`, `FASTFLAG*`, `FASTINT*`, `FASTSTRING`, `LOGGROUP`, `LOGVARIABLE`, `DYNAMIC_*`
- `RBX::VistaAPIs` (`isVistaOrBetter`, `SHGetKnownFolderPath`), `fixExceptionsThroughKernel`, `convert_w2s`, `convert_s2w`, `utf8_encode`, `utf8_decode`, `safeToLower`, `simple_logger`

### 🔴 Removed-only engine symbols (anti-cheat / VMProtect / integrity)
- `RBX::Security::patchMain`, `generateNetPmcKeys`, `teaEncrypt`, `setHackFlagVs`, `hackFlag6`, `NetPmcChallenge`, `kChallenges`, `kNumChallenges`, and every `RBX::Security::rbx{Text,Rdata,Iat,Vmp,Gold,...}{Base,Size,...}` global
- `RBX::Hasher::kGold*/kRdata*/kVmp*` constants, `RBX::ProgramMemoryChecker`, `RBX::pmcHash`, `RBX::Hasher`
- `RBX::Tokens::{sendStatsToken,simpleToken,apiToken}` (`addFlagSafe/addFlagFast`), `HATE_*` hack-flag macros
- `RBX::isSandboxie`, `CollectMd5Hash`, `vmProtectedDetectCheatEngineIcon` (`util/CheatEngine.h`), `setupCeLogWatcher`
- `RBX::hotpatchUnhook`/`hotpatchHook`/`hookingApiHooked` (defined in the removed `functionHooks.*`), `RBX::writecopyTrap`, `RBX::vehHookLocationHv`, `RBX::vehStubLocationHv`, `vehHookContinue`, `rbxNtdllProcAddress`, `netPmcHashCheck`
- `RBX::Security::FuzzyTokens`, `RBX::Security::ApiSecurity`, `Security/RandomConstant.h`, `Security/JunkCode.h` (`junk<>`), `RBX_BUILDSEED`

---

## 2. VMProtect SDK 🔴 (entirely removed)
`VMProtectBeginMutation`, `VMProtectEnd`, `VMProtectIsDebuggerPresent`,
`VMProtectFree`, `VMProtect/VMProtectSDK.h`.

---

## 3. Win32 / CRT / platform SDK

### Windowing & messages
`RegisterClassEx`, `CreateWindow`, `DefWindowProc`, `ShowWindow`,
`ShowWindowAsync`, `UpdateWindow`, `DestroyWindow`, `GetMessage`,
`TranslateMessage`, `DispatchMessage`, `PeekMessage`, `PostMessage`,
`SendMessage`, `PostQuitMessage`, `SetTimer`, `LoadIcon`, `LoadCursor`,
`LoadString`/`LoadStringA`, `LoadAccelerators`, `CopyAcceleratorTable`,
`SetWindowLongPtr`/`GetWindowLong`, `SetWindowPos`, `GetWindowPlacement`/
`SetWindowPlacement`, `GetWindowRect`, `GetClientRect`, `MoveWindow`,
`FindWindow`, `GetParent`, `SetFocus`, `MessageBox`/`MessageBoxA`,
`SetClassLong`, `MAKEINTRESOURCE`, `MapVirtualKeyEx`, `ToAsciiEx`,
`GetKeyboardLayout`, `GetKeyboardState`, `TrackMouseEvent`, `GET_X/Y_LPARAM`,
`LOWORD`/`HIWORD`.

### Monitors / display
`MonitorFromWindow`, `GetMonitorInfo`, `EnumDisplaySettingsEx`,
`ChangeDisplaySettingsEx`, `GetSystemMetrics`, `SystemParametersInfo`,
`WriteProfileString`/`GetProfileString`.

### Threads / sync / processes / memory
`CreateEvent`/`CreateEventA`, `OpenEventA`, `SetEvent`, `ResetEvent`,
`CreateMutexA`, `ReleaseMutex`, `WaitForSingleObject`,
`WaitForMultipleObjects`, `CreateProcess`, `ResumeThread`,
`GetCurrentThreadId`, `GetCurrentProcess`, `InterlockedIncrement`/`Decrement`,
`CreateFileMapping`, `MapViewOfFile`/`UnmapViewOfFile`, `CloseHandle`,
`CopyMemory`/`ZeroMemory`/`memset`/`memcmp`/`memcpy`,
`VirtualProtect` 🔴, `WriteProcessMemory` 🔴, `ReadProcessMemory` 🔴,
`GetModuleInformation` 🔴 (psapi).

### Files / time / locale / power / net
`GetModuleHandle`/`A`/`W`, `GetModuleFileName`/`W`, `GetProcAddress`,
`SetCurrentDirectoryW`, `FindFirstFile`/`FindNextFile`, `DeleteFile`,
`GetTempPath`, `GetSystemTime`, `SystemTimeToFileTime`,
`FileTimeToLocalFileTime`/`SystemTime`, `GetUserGeoID`/`GetGeoInfo`,
`IsNetworkAlive` (sensapi), `SetProcessDEPPolicy` (kept), `OutputDebugString`,
`GetLastError`, `Sleep`, `ShellExecute`/`ShellExecuteW`,
`CoTaskMemFree`, `SHGetKnownFolderPath`.

### COM / OLE / ActiveX / IE host
`CoInitializeEx`/`CoUninitialize`, `IUnknown`/`IDispatch`
(`QueryInterface`/`AddRef`/`Release`/`GetTypeInfo*`/`GetIDsOfNames`/`Invoke`),
`IConnectionPointContainer`/`IConnectionPoint` (`FindConnectionPoint`,
`Advise`), `IDocHostUIHandler`, `IOleClientSite`, `ICustomDoc::SetUIHandler`,
`SHDocVw::IWebBrowserApp`/`IWebBrowser2` (`Navigate`, `get_Document`,
`put_Width/Height`), `DWebBrowserEvents`/`2`, `UrlMkSetSessionOption`,
`DISPID_*` ids, `_bstr_t`, `_variant_t`, `VARIANT`.

### 🔴 Authenticode / crypto (removed with `Crypt.cpp`)
`WinVerifyTrust`, `CryptQueryObject`, `CryptMsgGetParam`, `CryptMsgClose`,
`CryptDecodeObject`, `CertFindCertificateInStore`, `CertGetNameString`,
`CertFreeCertificateContext`, `CertCloseStore`, `LocalAlloc`/`LocalFree`
(in that file), `StrCmpW`, `WINTRUST_*` structs.

### DirectInput (dinput8)
`DirectInput8Create`, `IDirectInput8::CreateDevice`,
`IDirectInputDevice8::{SetDataFormat,SetProperty,SetCooperativeLevel,Acquire,
Unacquire,GetDeviceData,GetDeviceState}`, `c_dfDIMouse2`, `c_dfDIKeyboard`,
`GUID_SysMouse`, `GUID_SysKeyboard`.

### ATL / MFC
`CWindowImpl`, `CAxDialogImpl`, `CComModule`, `CComPtr`/`CComQIPtr`,
`CCriticalSection`/`CCritSecLock`, `CEvent`, `CRegKey`
(`Open`/`QueryDWORDValue`/`QueryStringValue`), `CUrl`
(`CrackUrl`/`GetHostName`), `CString`, `CRect`, `_AtlBaseModule`,
`CenterWindow`, `DoModal`, `EndDialog`, `GetDlgControl`, `SetIcon`,
`DECLARE_WND_CLASS`, `BEGIN/END_MSG_MAP`, `SAFE_STATIC`.

---

## 4. Third-party libraries

### Boost
`boost::program_options` (`options_description`, `variables_map`, `store`,
`command_line_parser`, `split_winmain`), `boost::thread`/`recursive_mutex`/
`this_thread::sleep`, `boost::bind`, `boost::function`, `boost::scoped_ptr`/
`shared_ptr`/`weak_ptr`/`make_shared_future`, `boost::format`,
`boost::filesystem` (`path`, `temp_directory_path`, `copy_file`, `remove`,
`rename`), `boost::replace_all`, `boost::posix_time`.

### G3D
`G3D::System::{hasSSE2,time}`, `G3D::Vector2`/`Vector2int16`/`Vector3`/
`Vector4`/`Rect2D`, `Math::expandVector2`.

### C/C++ runtime
`rand`, `atoi`/`atof`, `itoa`, `tolower`, `std::transform`, `std::min/max`,
`std::random_shuffle`, `_strnicmp`, `wcscmp`/`wcslen`, `strncmp`/`strcpy_s`/
`strnlen`, `lstrcpynW`/`lstrcpyW`, `std::ifstream`/`ofstream`/`stringstream`,
`_ReturnAddress` 🔴.

---

### Files removed wholesale (all their externals are 🔴)
`functionHooks.cpp/.h`, `robloxHooks.cpp` + `RobloxHooks.h`,
`RandomPadding.cpp`, `ReleasePatcher.cpp/.h`, `Crypt.cpp/.h`.
