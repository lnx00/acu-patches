use windows::Win32::Foundation::HMODULE;

pub struct ImGuiShared;
pub struct ACUPluginLoaderSharedGlobals;

pub const fn make_version(major: u64, minor: u64, minorer: u64, minorest: u64) -> u64 {
    (major << 24) | (minor << 16) | (minorer << 8) | minorest
}

/// Interface provided by the plugin loader to plugins
#[repr(C)]
pub struct ACUPluginLoaderInterface {
    /// Version of the plugin loader API
    pub m_plugin_loader_version: u64,

    /// Call this if you want the PluginLoader to unload this DLL
    pub request_unload_plugin: Option<unsafe extern "C" fn(dll_handle: HMODULE)>,

    /// Can be used for very basic interaction between plugins
    pub get_plugin_if_loaded: Option<unsafe extern "C" fn(plugin_name: *const u16) -> HMODULE>,

    /// Shared variables from the implementation
    pub m_implementation_shared_variables: *mut ACUPluginLoaderSharedGlobals,
}

#[repr(C)]
pub struct ACUPluginInfo {
    pub m_plugin_api_version: u64,
    pub m_plugin_version: u64,

    /// Called when Main Integrity Check is disabled and code patches are safe to apply
    /// Return `false` to unload the plugin.
    pub m_init_stage_when_code_patches_are_safe_to_apply:
        Option<extern "C" fn(plugin_loader: &ACUPluginLoaderInterface) -> bool>,

    /// Called every frame when the menu section for _your_ plugin is visible
    pub m_every_frame_when_menu_is_open: Option<extern "C" fn(imgui_context: &ImGuiShared)>,

    /// Called every frame
    pub m_every_frame_even_when_menu_is_closed: Option<extern "C" fn(imgui_context: &ImGuiShared)>,

    /// Called after API version compatibility check
    /// This is very early during game load
    pub m_init_stage_when_versions_are_deemed_compatible:
        Option<extern "C" fn(plugin_loader: &ACUPluginLoaderInterface)>,

    /// Early hook that runs before MainIntegrityCheck is killed
    pub m_early_hook_when_game_code_is_unpacked: Option<extern "C" fn()>,
}
