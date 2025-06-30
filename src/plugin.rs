use std::{
    ffi::{CString, c_char},
    sync::OnceLock,
};

use assert_offset::AssertOffsets;
use log::{LevelFilter, Log, Metadata, Record};
use windows::Win32::Foundation::HMODULE;

pub struct ImGuiShared;
pub struct InputHooks;
pub struct AnimationModdingInterface;

pub const fn make_version(major: u64, minor: u64, minorer: u64, minorest: u64) -> u64 {
    (major << 24) | (minor << 16) | (minorer << 8) | minorest
}

pub const PLUGIN_API_VERSION: u64 = make_version(0, 9, 1, 0);
const PKG_NAME: Option<&str> = option_env!("CARGO_PKG_NAME");

#[derive(AssertOffsets)]
#[repr(C)]
pub struct ImGuiConsoleInterface {
    #[offset(0x0)]
    pub fnp_add_log: Option<unsafe extern "C" fn(s: *const c_char)>,
}

impl ImGuiConsoleInterface {
    pub fn add_log(&self, text: &str) {
        if let Some(log_fn) = self.fnp_add_log {
            let c_string = CString::new(text);
            if let Ok(str) = c_string {
                unsafe {
                    log_fn(str.as_ptr());
                }
            }
        }
    }
}

static IMGUI_CONSOLE: OnceLock<&'static ImGuiConsoleInterface> = OnceLock::new();

struct ImGuiLogger;

impl Log for ImGuiLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let formatted = format!("[{}][{}] {}", PKG_NAME.unwrap_or("???"), record.level(), record.args());
        println!("{}", formatted);

        if let Some(console) = IMGUI_CONSOLE.get() {
            console.add_log(&formatted);
        }
    }

    fn flush(&self) {}
}

#[derive(AssertOffsets)]
#[repr(C)]
pub struct ACUPluginLoaderSharedGlobals {
    #[offset(0x0)]
    pub input_hooks: *mut InputHooks,

    #[offset(0x8)]
    pub animation_modding: *mut AnimationModdingInterface,

    #[offset(0x10)]
    pub imgui_console: *mut ImGuiConsoleInterface,
}

/// Interface provided by the plugin loader to plugins
#[derive(AssertOffsets)]
#[repr(C)]
pub struct ACUPluginLoaderInterface {
    /// Version of the plugin loader API
    #[offset(0x0)]
    pub m_plugin_loader_version: u64,

    /// Call this if you want the PluginLoader to unload this DLL
    #[offset(0x8)]
    pub request_unload_plugin: Option<unsafe extern "C" fn(dll_handle: HMODULE)>,

    /// Can be used for very basic interaction between plugins
    #[offset(0x10)]
    pub get_plugin_if_loaded: Option<unsafe extern "C" fn(plugin_name: *const u16) -> HMODULE>,

    /// Shared variables from the implementation
    #[offset(0x18)]
    pub m_implementation_shared_variables: *mut ACUPluginLoaderSharedGlobals,
}

impl ACUPluginLoaderInterface {
    pub fn init_logger(&self) -> Result<(), log::SetLoggerError> {
        if let Some(console) = unsafe {
            self.m_implementation_shared_variables
                .as_ref()
                .and_then(|globals| globals.imgui_console.as_ref())
        } {
            let _ = IMGUI_CONSOLE.set(console);
        }

        log::set_boxed_logger(Box::new(ImGuiLogger))?;
        log::set_max_level(LevelFilter::Debug);

        Ok(())
    }
}

#[derive(AssertOffsets)]
#[repr(C)]
pub struct ACUPluginInfo {
    #[offset(0x0)]
    pub m_plugin_api_version: u64,

    #[offset(0x8)]
    pub m_plugin_version: u64,

    /// Called when Main Integrity Check is disabled and code patches are safe to apply
    /// Return `false` to unload the plugin.
    #[offset(0x10)]
    pub m_init_stage_when_code_patches_are_safe_to_apply:
        Option<extern "C" fn(plugin_loader: &ACUPluginLoaderInterface) -> bool>,

    /// Called every frame when the menu section for _your_ plugin is visible
    #[offset(0x18)]
    pub m_every_frame_when_menu_is_open: Option<extern "C" fn(imgui_context: &ImGuiShared)>,

    /// Called every frame
    #[offset(0x20)]
    pub m_every_frame_even_when_menu_is_closed: Option<extern "C" fn(imgui_context: &ImGuiShared)>,

    /// Called after API version compatibility check
    /// This is very early during game load
    #[offset(0x28)]
    pub m_init_stage_when_versions_are_deemed_compatible:
        Option<extern "C" fn(plugin_loader: &ACUPluginLoaderInterface)>,

    /// Early hook that runs before MainIntegrityCheck is killed
    #[offset(0x30)]
    pub m_early_hook_when_game_code_is_unpacked: Option<extern "C" fn()>,
}
