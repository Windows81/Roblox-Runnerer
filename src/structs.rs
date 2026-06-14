use ::std::ffi::CString;
use std::ffi::c_void;

#[repr(C)]
pub struct std_string {
    _unknown: [u8; 0x20],
}

#[repr(C)]
struct ref_count_base {
    vftable: usize,
    pub uses: u32,
    pub weaks: u32,
}

#[repr(C)]
pub struct shared_ptr<T> {
    pub ptr: *mut T,
    pub rep: *mut ref_count_base,
}

#[repr(C)]
struct rbx_signal
// sizeof=0x18
{
    head: usize,
    slots: shared_ptr<c_void>,
}

#[repr(C)]
pub struct OSContext
// sizeof=0x20
{
    pub hwnd: usize,
    pub vr_enabled: bool,
    pub vr_context: usize,
    pub screen_dpiscale: f32,
}

/* 1235 */
#[repr(C)]
pub enum GraphicsMode {
    UnknownGraphicsMode = 0x0,
    AutoGraphicsMode = 0x1,
    Direct3D11 = 0x2,
    // removed_Direct3D9 = 0x3,
    Metal = 0x5,
    Vulkan = 0x6,
    NoGraphics = 0x7,
}

/* 1236 */
#[repr(C)]
pub enum FrameRateManagerMode {
    FrameRateManagerAuto = 0x0,
    FrameRateManagerOn = 0x1,
    FrameRateManagerOff = 0x2,
}

/* 1237 */
#[repr(C)]
pub enum QualityLevel {
    QualityAuto = 0x0,
    QualityLevel1 = 0x1,
    QualityLevel2 = 0x2,
    QualityLevel3 = 0x3,
    QualityLevel4 = 0x4,
    QualityLevel5 = 0x5,
    QualityLevel6 = 0x6,
    QualityLevel7 = 0x7,
    QualityLevel8 = 0x8,
    QualityLevel9 = 0x9,
    QualityLevel10 = 0xA,
    QualityLevel11 = 0xB,
    QualityLevel12 = 0xC,
    QualityLevel13 = 0xD,
    QualityLevel14 = 0xE,
    QualityLevel15 = 0xF,
    QualityLevel16 = 0x10,
    QualityLevel17 = 0x11,
    QualityLevel18 = 0x12,
    QualityLevel19 = 0x13,
    QualityLevel20 = 0x14,
    QualityLevel21 = 0x15,
    QualityLevelCount = 0x16,
}

#[repr(C)]
pub enum MeshPartDetailLevel {
    DistanceBased = 0x0,
    UseMeshPartDetailLOD0 = 0x1,
    UseMeshPartDetailLOD1 = 0x2,
    UseMeshPartDetailLOD2 = 0x3,
    UseMeshPartDetailLOD3 = 0x4,
    UseMeshPartDetailLOD4 = 0x5,
}

#[repr(C)]
pub struct Vector2int16 {
    pub x: i16,
    pub y: i16,
}

#[repr(C)]
pub struct CRenderSettings {
    pub __vftable: usize,
    pub graphics_mode: GraphicsMode,
    pub frame_rate_manager_mode: FrameRateManagerMode,
    pub quality_level: QualityLevel,
    pub edit_quality_level: QualityLevel,
    pub debug_mesh_detaillevel: MeshPartDetailLevel,
    pub auto_quality_level: i32,
    pub max_quality_level: i32,
    pub debug_show_bounding_boxes: bool,
    pub debug_reload_assets: bool,
    pub enable_frm: bool,
    pub obj_export_merge_by_material: bool,
    pub fullscreen_size: Vector2int16,
    pub window_size: Vector2int16,
    pub eager_bulk_execution: bool,
    pub mesh_cache_size: u32,
}

/// `SharedLauncher::LaunchMode`
#[repr(C)]
pub enum LaunchMode {
    Play,
    PlayProtocol,
    App,
}

/// `DataModelJob::*` access kind for `submitTask` / locks.
#[repr(C)]
pub enum JobAccess {
    Read,
    Write,
    Render,
}

/// `Security::*` impersonation identity.
#[repr(C)]
pub enum SecurityIdentity {
    Com,
    RobloxGameScript,
}

/// `TaskScheduler::StepResult`.
#[repr(C)]
pub enum StepResult {
    Done,
    Stepped,
}

/// `Network::GameLaunchIntent::*` impersonation identity.
#[repr(C)]
pub enum GameLaunchIntent {
    GameLaunchIntentUnknown = 0x0,
    GameLaunchIntentClient = 0x1,
    GameLaunchIntentServer = 0x2,
    GameLaunchIntentNonNetworked = 0x3,
}

pub struct Verb;
pub struct DataModel;
pub struct PlayerConfigurer;
pub struct CommonVerbs;
pub struct ViewBase;

/* 85 */
#[repr(C)]
pub enum StudioGameStateType {
    StudioGameStateTypeEdit = 0x0,
    StudioGameStateTypePlayClient = 0x1,
    StudioGameStateTypePlayServer = 0x2,
    StudioGameStateTypeStandalone = 0x3,
    NumStudioGameStateTypes = 0x4,
}

/* 568 */
#[repr(C)]
pub enum DataModelFeatures {
    ClientReplicator = 0x1,
    ServerReplicator = 0x2,
    ScriptExecution = 0x8,
    Graphics = 0x10,
    Gui = 0x20,
    ConsideredOriginal = 0x40,
}

/* 36282 */
#[repr(C)]
pub struct DataModelInitializationParams {
    pub system_locale_id: std_string,
    pub roblox_locale_id: std_string,
    pub game_locale_id: std_string,
    pub force_roblox_locale_id_deprecated: std_string,
    pub force_game_locale_id_deprecated: std_string,
    pub force_dont_use_data_model_patch: bool,
    pub load_core_script_translations: bool,
    pub load_fonts: bool,
    pub is_cloud_edit: bool,
    pub screen_dpiscale: f32,
    pub place_id: i64,
    pub studio_game_state_type: StudioGameStateType,
    pub sound_enabled: bool,
    pub is_luobu_server: [u8; 0x02], // std::optional<bool>
    pub data_model_features: DataModelFeatures,
}

/* 40269 */
#[repr(C)]
pub struct Game {
    pub __vftable: usize,
    pub has_shutdown: bool,
    pub game_configurer: shared_ptr<PlayerConfigurer>,
    pub data_model: shared_ptr<DataModel>,
    _verbs: [usize; 3],
    pub common_verbs: shared_ptr<CommonVerbs>,
    pub signal_disconnected: rbx_signal,
    pub initialization_params: DataModelInitializationParams,
}
