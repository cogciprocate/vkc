//! vkc - Vulkan Compute

#![allow(unused_extern_crates, unused_imports, dead_code, unused_variables)]

extern crate libloading as lib;
extern crate smallvec;
pub extern crate vk_sys as vk;
pub extern crate winit;

mod version;
mod loader;
mod instance;
pub mod surface;
pub mod device;
pub mod queue;

use std::ffi::OsStr;
use std::os::raw::c_void;
use std::mem;
use std::ptr;
use winit::{EventsLoop, WindowBuilder, Window, CreationError, ControlFlow, Event, WindowEvent};
use loader::Loader;
pub use version::Version;
pub use instance::Instance;
pub use device::Device;
pub use surface::Surface;

pub type VkcResult<T> = Result<T, ()>;


fn check(code: u32) {
    if code != vk::SUCCESS { panic!("Error code: {}", code); }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {

    }
}
