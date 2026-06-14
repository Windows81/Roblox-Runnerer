use std::{ffi::CString, mem::transmute};

use crate::structs::{
    CRenderSettings, DataModelInitializationParams, Game, GameLaunchIntent, GraphicsMode,
    OSContext, Verb, ViewBase,
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
}

pub fn find(rōblox_dll_ptr: u64) -> DataModel {
    unsafe {
        DataModel::with_offset(
            // TODO: build dynamic offset finder for versions other than v548.
            &DataModel {
                unsecured_studio_game: transmute(0x12A5680 as usize),
                create_view: transmute(0x220ACA0 as usize),
            },
            rōblox_dll_ptr as _,
        )
    }
}
