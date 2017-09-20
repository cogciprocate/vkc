use std::ffi::OsStr;
use std::mem;
use std::ptr;
use std::fmt;
use lib;
use vks;
use vk::{self, PFN_vkGetInstanceProcAddr};
use ::{VkcResult, ENABLE_VALIDATION_LAYERS};

pub struct Loader {
    vk_lib: lib::Library,
    vk_get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
    // entry_points: vk::VkEntryPoints,
    loader: vks::InstanceProcAddrLoader,
}

impl Loader {
    pub fn new() -> VkcResult<Loader> {
        let lib_filename = if cfg!(not(any(target_os = "macos", target_os = "ios"))) {
            if cfg!(all(unix, not(target_os = "android"), not(target_os = "macos"))) { "libvulkan.so.1" }
            else if cfg!(target_os = "android") { "libvulkan.so" }
            else if cfg!(windows) { "vulkan-1.dll" }
            else { unimplemented!("unknown operating system") }
        } else {
            unimplemented!("macos not implemented");
        };
        let vk_lib = lib::Library::new(lib_filename).unwrap();

        let vk_get_instance_proc_addr = unsafe {
            let fn_name = "vkGetInstanceProcAddr";

            let get_proc_addr: lib::Symbol<vk::PFN_vkGetInstanceProcAddr> = vk_lib.get(fn_name.as_bytes()).unwrap();
            *get_proc_addr
        };

        let mut loader = vks::InstanceProcAddrLoader::from_get_instance_proc_addr(vk_get_instance_proc_addr);
        unsafe {
            loader.load_core_global();
        }

        Ok(Loader { vk_lib, vk_get_instance_proc_addr, loader })
    }

    #[inline]
    pub fn get_instance_proc_addr(&self, instance: vk::VkInstance, name: *const i8)
            -> Option<unsafe extern "system" fn(*mut vk::VkInstance_T, *const i8)
                -> Option<unsafe extern "system" fn()>>
    {
        self.vk_get_instance_proc_addr
    }

    #[inline]
    pub fn core_global(&self) -> &vks::instance_proc_addr_loader::CoreGlobal {
        &self.loader.core_global
    }

    #[inline]
    pub fn loader(&self) -> &vks::InstanceProcAddrLoader {
        &self.loader
    }

    #[inline]
    pub fn loader_mut(&mut self) -> &mut vks::InstanceProcAddrLoader {
        &mut self.loader
    }

}

impl fmt::Debug for Loader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Loader")
    }
}