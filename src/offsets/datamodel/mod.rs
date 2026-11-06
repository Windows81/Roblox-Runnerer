use std::{ffi::CString, mem::transmute};

use crate::{
    offsets::util::{CallClass, load_memory},
    structs::{
        CRenderSettings, DataModel, DataModelInitializationParams, Game, GameLaunchIntent,
        GraphicsMode, OsContext, SharedPtr, Verb, ViewBase,
    },
};

#[derive(Debug)]
#[repr(C)]
pub struct DataModelFns {
    /// ```cpp
    /// RBX::UnsecuredStudioGame *__fastcall RBX::UnsecuredStudioGame::UnsecuredStudioGame(
    ///    RBX::UnsecuredStudioGame *this,
    ///    RBX::Verb *lockVerb,
    ///    const char *baseUrl,
    ///    bool isNetworked,
    ///    bool shouldShowLoadingScreen,
    ///    RBX::Network::GameLaunchIntent intent,
    ///    RBX::DataModelInitializationParams *initializationParams)
    /// ```
    pub unsecured_studio_game: extern "C" fn(
        this: Game,
        lock_verb: Verb,
        base_url: CString,
        is_networked: bool,
        show_loading_screen: bool,
        intent: GameLaunchIntent,
        params: DataModelInitializationParams,
    ) -> Game,

    /// ```cpp
    /// RBX::ViewBase *__fastcall RBX::ViewBase::CreateView(
    ///    RBX::CRenderSettings::GraphicsMode mode,
    ///    RBX::OSContext *context,
    ///    RBX::CRenderSettings *renderSettings)
    /// ```
    pub create_view: extern "C" fn(
        mode: GraphicsMode,
        context: OsContext,
        render_settings: CRenderSettings,
    ) -> ViewBase,

    /// ```cpp
    /// std::shared_ptr<RBX::DataModel> *__fastcall RBX::DataModel::createDataModel(
    ///    std::shared_ptr<RBX::DataModel> *result,
    ///    bool startHeartbeat,
    ///    RBX::Verb *lockVerb,
    ///    bool shouldShowLoadingScreen,
    ///    RBX::Network::GameLaunchIntent intent,
    ///    const RBX::DataModelInitializationParams *initializationParams)
    /// ```
    pub create_data_model: extern "C" fn(
        result: SharedPtr<DataModel>,
        start_heartbeat: bool,
        lock_verb: Verb,
        show_loading_screen: bool,
        intent: GameLaunchIntent,
        params: DataModelInitializationParams,
    ) -> SharedPtr<DataModel>,
}

impl CallClass for DataModelFns {}

// The RVAs below are only valid for this exact build.
const SUPPORTED_VERSION: &[u8] = b"0.548.488.5480525";
const VERSION_STRING_RVA: u64 = 0x3C0_68A0;

pub fn find(base: u64) -> DataModelFns {
    let found = load_memory(base + VERSION_STRING_RVA, SUPPORTED_VERSION.len() as _);
    assert_eq!(found, SUPPORTED_VERSION, "unsupported Studio build");

    // TODO: build dynamic offset finder for versions other than v548.
    unsafe {
        let rvas = DataModelFns {
            unsecured_studio_game: transmute(0x0BD_98E0_usize),
            create_view: transmute(0x220_ACA0_usize),
            create_data_model: transmute(0x12A_5680_usize),
        };
        rvas.with_offset(base as _)
    }
}
