//! vkc - Vulkan Compute

#![allow(unused_extern_crates, unused_imports, dead_code, /*unused_variables*/)]

extern crate libloading as lib;
extern crate smallvec;
pub extern crate vk_sys as vk;
pub extern crate winit;

mod version;
mod loader;
mod instance;
mod swapchain;
mod image_views;
// pub mod vulkan_h;
mod pipeline_layout;
mod shader_module;
mod render_pass;
mod graphics_pipeline;
pub mod surface;
pub mod device;
pub mod queue;
pub mod util;

use std::ffi::OsStr;
use std::os::raw::c_void;
use std::mem;
use std::ptr;
use winit::{EventsLoop, WindowBuilder, Window, CreationError, ControlFlow, Event, WindowEvent};
use loader::Loader;
// pub use vulkan_h as vk;
pub use version::Version;
pub use instance::Instance;
pub use device::Device;
pub use surface::Surface;
pub use swapchain::{Swapchain, SwapchainSupportDetails};
pub use image_views::{create_image_views, ImageView};
pub use shader_module::ShaderModule;
pub use pipeline_layout::PipelineLayout;
pub use render_pass::RenderPass;
pub use graphics_pipeline::GraphicsPipeline;

pub type VkcResult<T> = Result<T, ()>;


///////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////// TEMPLATE /////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////
// use std::sync::Arc;
// use std::ptr;
// use vk;
// use ::{util, Device};

// struct Inner {
//     handle: vk::AbstractTemplate,
//     device: Device,
// }

// pub struct AbstractTemplate {
//     inner: Arc<Inner>,
// }

// impl AbstractTemplate {
//     pub fn new() -> AbstractTemplate {
//         AbstractTemplate {
//             inner: Arc::new(Inner {
//                 handle,
//                 device,
//             })
//         }
//     }

//     pub fn handle(&self) -> vk::AbstractTemplate {
//         self.inner.handle
//     }

//     pub fn device(&self) -> &Device {
//         &self.inner.device
//     }
// }

// impl Drop for Inner {
//     fn drop(&mut self) {
//         unsafe {
//             self.device.vk().DestroyAbstractTemplate(self.device.handle(), self.handle, ptr::null());
//         }
//     }
// }
///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////



fn check(code: u32) {
    if code != vk::SUCCESS { panic!("Error code: {}", code); }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {

    }
}
