use std::ffi::OsStr;
use std::mem;
use std::ptr;
use std::fmt;
use lib;
use vk;


type ProcAddrFnSym<'lib> = lib::Symbol<'lib, extern "system" fn(instance: vk::Instance, pName: *const i8)
    -> extern "system" fn() -> ()>;

type ProcAddrFnRaw = extern "system" fn(instance: vk::Instance, pName: *const i8)
    -> extern "system" fn() -> ();


pub struct Loader {
    vk_lib: lib::Library,
    get_proc_addr: ProcAddrFnRaw,
    entry_points: vk::EntryPoints,
}

impl Loader {
    pub unsafe fn new() -> Loader {
        let lib_filename = if cfg!(not(any(target_os = "macos", target_os = "ios"))) {
            if cfg!(all(unix, not(target_os = "android"), not(target_os = "macos"))) { "libvulkan.so.1" }
            else if cfg!(target_os = "android") { "libvulkan.so" }
            else if cfg!(windows) { "vulkan-1.dll" }
            else { unimplemented!("unknown operating system") }
        } else {
            unimplemented!("macos not implemented");
        };

        let vk_lib = lib::Library::new(lib_filename).unwrap();
        let fn_name = b"vkGetInstanceProcAddr";
        let get_proc_addr: ProcAddrFnRaw = {
            let get_proc_addr: lib::Symbol<ProcAddrFnSym> = vk_lib.get(&fn_name[..]).unwrap();
            mem::transmute(get_proc_addr)
        };
        let entry_points = vk::EntryPoints::load(|name|
            mem::transmute((get_proc_addr)(0, name.as_ptr())));
        Loader { vk_lib, get_proc_addr, entry_points }
    }

    #[inline]
    pub fn get_instance_proc_addr(&self, instance: vk::Instance, name: *const i8)
            -> extern "system" fn() -> ()
    {
        (self.get_proc_addr)(instance, name)
    }

    #[inline]
    pub fn entry_points(&self) -> &vk::EntryPoints {
        &self.entry_points
    }

}

impl fmt::Debug for Loader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Loader")
    }
}