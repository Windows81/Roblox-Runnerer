use std::ffi::c_void;

#[repr(C)]
pub struct StdString {
    _unknown: [u8; 0x20],
}

#[repr(C)]
struct RefCountBase {
    vftable: usize,
    pub uses: u32,
    pub weaks: u32,
}

#[repr(C)]
pub struct SharedPtr<T> {
    pub ptr: *mut T,
    pub rep: *mut RefCountBase,
}

#[repr(C)]
pub struct IfStream {
    _unknown: [u8; 0x110],
}

#[repr(C)]
struct RbxSignal { // sizeof=0x18
    head: usize,
    slots: SharedPtr<c_void>,
}

#[repr(C)]
pub struct OsContext { // sizeof=0x20
    pub hwnd: usize,
    pub vr_enabled: bool,
    pub vr_context: usize,
    pub screen_dpiscale: f32,
}

#[repr(C)]
pub enum GraphicsMode {
    Unknown   = 0x0,
    Auto      = 0x1,
    Direct3D11 = 0x2,
    // removed: Direct3D9 = 0x3
    Metal     = 0x5,
    Vulkan    = 0x6,
    NoGraphics = 0x7,
}

#[repr(C)]
pub enum FrameRateManagerMode {
    Auto = 0x0,
    On   = 0x1,
    Off  = 0x2,
}

#[repr(C)]
pub enum QualityLevel {
    Auto    = 0x0,
    Level1  = 0x1,
    Level2  = 0x2,
    Level3  = 0x3,
    Level4  = 0x4,
    Level5  = 0x5,
    Level6  = 0x6,
    Level7  = 0x7,
    Level8  = 0x8,
    Level9  = 0x9,
    Level10 = 0xA,
    Level11 = 0xB,
    Level12 = 0xC,
    Level13 = 0xD,
    Level14 = 0xE,
    Level15 = 0xF,
    Level16 = 0x10,
    Level17 = 0x11,
    Level18 = 0x12,
    Level19 = 0x13,
    Level20 = 0x14,
    Level21 = 0x15,
    Count   = 0x16,
}

#[repr(C)]
pub enum MeshPartDetailLevel {
    DistanceBased = 0x0,
    Lod0 = 0x1,
    Lod1 = 0x2,
    Lod2 = 0x3,
    Lod3 = 0x4,
    Lod4 = 0x5,
}

#[repr(C)]
pub struct Vector2Int16 {
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
    pub fullscreen_size: Vector2Int16,
    pub window_size: Vector2Int16,
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

#[repr(C)]
pub enum GameLaunchIntent {
    Unknown      = 0x0,
    Client       = 0x1,
    Server       = 0x2,
    NonNetworked = 0x3,
}

pub struct Verb;
pub struct DataModel;
pub struct PlayerConfigurer;
pub struct CommonVerbs;
pub struct ViewBase;

#[repr(C)]
pub enum StudioGameStateType {
    Edit       = 0x0,
    PlayClient = 0x1,
    PlayServer = 0x2,
    Standalone = 0x3,
    Count      = 0x4,
}

#[repr(C)]
pub enum DataModelFeatures {
    ClientReplicator  = 0x1,
    ServerReplicator  = 0x2,
    ScriptExecution   = 0x8,
    Graphics          = 0x10,
    Gui               = 0x20,
    ConsideredOriginal = 0x40,
}

#[repr(C)]
pub struct DataModelInitializationParams {
    pub system_locale_id: StdString,
    pub roblox_locale_id: StdString,
    pub game_locale_id: StdString,
    pub force_roblox_locale_id_deprecated: StdString,
    pub force_game_locale_id_deprecated: StdString,
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

#[repr(C)]
pub struct Game {
    pub __vftable: usize,
    pub has_shutdown: bool,
    pub game_configurer: SharedPtr<PlayerConfigurer>,
    pub data_model: SharedPtr<DataModel>,
    _verbs: [usize; 3],
    pub common_verbs: SharedPtr<CommonVerbs>,
    pub signal_disconnected: RbxSignal,
    pub initialization_params: DataModelInitializationParams,
}
