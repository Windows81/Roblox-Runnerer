use std::{ffi::CString, mem::transmute};

use crate::{
    offsets::util::load_memory,
    structs::{
        CRenderSettings, DataModelInitializationParams, Game, GameLaunchIntent, GraphicsMode,
        OSContext, Verb, ViewBase, shared_ptr,
    },
};

use super::util::CallClass;

impl CallClass for DataModel {}
#[derive(Debug)]
#[repr(C)]
pub struct DataModel {
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
        lockVerb: Verb,
        baseUrl: CString,
        isNetworked: bool,
        shouldShowLoadingScreen: bool,
        intent: GameLaunchIntent,
        initializationParams: DataModelInitializationParams,
    ) -> Game,

    /// ```cpp
    /// RBX::ViewBase *__fastcall RBX::ViewBase::CreateView(
    ///    RBX::CRenderSettings::GraphicsMode mode,
    ///    RBX::OSContext *context,
    ///    RBX::CRenderSettings *renderSettings)
    /// ```
    pub create_view: extern "C" fn(
        mode: GraphicsMode,
        context: OSContext,
        renderSettings: CRenderSettings,
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
        this: shared_ptr<DataModel>,
        startHeartbeat: bool,
        lockVerb: Verb,
        shouldShowLoadingScreen: bool,
        intent: GameLaunchIntent,
        initializationParams: DataModelInitializationParams,
    ) -> shared_ptr<DataModel>,
}

pub fn find(rōblox_dll_ptr: u64) -> DataModel {
    const TEST_VERSION_STR: &[u8] = "0.548.488.5480525".as_bytes();
    assert_eq!(
        load_memory(rōblox_dll_ptr + 0x3c068a0, TEST_VERSION_STR.len() as _),
        TEST_VERSION_STR
    );
    unsafe {
        let base=
            // TODO: build dynamic offset finder for versions other than v548.
            &DataModel {
                unsecured_studio_game: transmute(0x0BD98E0 as usize),
                create_view: transmute(0x220ACA0 as usize),
                create_data_model: transmute(0x12A5680 as usize),
            };
        DataModel::with_offset(base, rōblox_dll_ptr as _)
    }
}
