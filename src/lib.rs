//! vkc - Vulkan Compute

#![allow(unused_extern_crates, unused_imports, dead_code, unused_variables)]

extern crate libloading as lib;
extern crate smallvec;
// extern crate nalgebra;
extern crate vks;
extern crate libc;
pub extern crate winit;

mod version;
mod loader;
mod instance;
mod swapchain;
mod image_view;
mod pipeline_layout;
mod shader_module;
mod render_pass;
mod graphics_pipeline;
mod framebuffer;
mod surface;
mod queue;
mod command_pool;
mod command_buffers;
mod semaphore;
mod buffer;
mod device_memory;

pub mod vk {
    pub use vks::*;
    pub use vks::core::*;
    pub use vks::amd_rasterization_order::*;
    pub use vks::ext_debug_marker::*;
    pub use vks::ext_debug_report::*;
    pub use vks::ext_validation_flags::*;
    pub use vks::khr_android_surface::*;
    pub use vks::khr_display::*;
    pub use vks::khr_display_swapchain::*;
    pub use vks::khr_get_physical_device_properties2::*;
    pub use vks::khr_mir_surface::*;
    pub use vks::khr_surface::*;
    pub use vks::khr_swapchain::*;
    pub use vks::khr_wayland_surface::*;
    pub use vks::khr_win32_surface::*;
    pub use vks::khr_xcb_surface::*;
    pub use vks::khr_xlib_surface::*;
    pub use vks::nv_dedicated_allocation::*;
    pub use vks::nv_external_memory::*;
    pub use vks::nv_external_memory_capabilities::*;
    pub use vks::nv_external_memory_win32::*;
    pub use vks::nv_win32_keyed_mutex::*;
}

// pub mod vulkan_h;
pub mod device;
pub mod util;

use std::ffi::OsStr;
use libc::c_void;
use std::mem;
use std::ptr;
use winit::{EventsLoop, WindowBuilder, Window, CreationError, ControlFlow, Event, WindowEvent};
use loader::Loader;
// pub use vulkan_h as vk;
// pub use vks::core as vkscore;
// use vk::*;
pub use version::Version;
pub use instance::Instance;
pub use device::Device;
pub use surface::Surface;
pub use queue::{queue_families, Queue};
pub use swapchain::{Swapchain, SwapchainSupportDetails};
pub use image_view::{create_image_views, ImageView};
pub use shader_module::ShaderModule;
pub use pipeline_layout::PipelineLayout;
pub use render_pass::RenderPass;
pub use graphics_pipeline::GraphicsPipeline;
pub use framebuffer::{create_framebuffers, Framebuffer};
pub use command_pool::CommandPool;
pub use command_buffers::create_command_buffers;
pub use semaphore::Semaphore;
pub use buffer::Buffer;
pub use device_memory::DeviceMemory;

pub type VkcResult<T> = Result<T, ()>;

#[cfg(debug_assertions)]
pub const ENABLE_VALIDATION_LAYERS: bool = true;
#[cfg(not(debug_assertions))]
pub const ENABLE_VALIDATION_LAYERS: bool = false;


#[macro_export]
macro_rules! offset_of {
    ($ty:ty, $field:ident) => {
        unsafe { &(*(0 as *const $ty)).$field as *const _ as usize } as u32
    }
}

#[repr(C)]
pub struct Vertex {
    pos: [f32; 2],
    color: [f32; 3],
}

impl Vertex {
    pub fn binding_description() -> vk::VkVertexInputBindingDescription {
        vk::VkVertexInputBindingDescription {
            binding: 0,
            stride: mem::size_of::<Vertex>() as u32,
            // * VERTEX_INPUT_RATE_VERTEX: Move to the next data entry
            //   after each vertex
            // * VERTEX_INPUT_RATE_INSTANCE: Move to the next data entry
            //   after each instance
            inputRate: vk::VK_VERTEX_INPUT_RATE_VERTEX,
        }
    }

    pub fn attribute_descriptions() -> [vk::VkVertexInputAttributeDescription; 2] {
        [
            vk::VkVertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vk::VK_FORMAT_R32G32_SFLOAT,
                offset: offset_of!(Vertex, pos),
            },
            vk::VkVertexInputAttributeDescription {
                location: 1,
                binding: 0,
                format: vk::VK_FORMAT_R32G32B32_SFLOAT,
                offset: offset_of!(Vertex, color),
            },
        ]
    }
}

pub const VERTICES: [Vertex; 3] =  [
    Vertex { pos: [0.0f32, -0.5f32], color: [1.0f32, 0.0f32, 0.0f32] },
    Vertex { pos: [0.5f32, 0.5f32], color: [0.0f32, 1.0f32, 0.0f32] },
    Vertex { pos: [-0.5f32, 0.5f32], color: [0.0f32, 0.0f32, 1.0f32] },
];





///////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////// TEMPLATE /////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////
// use std::sync::Arc;
// use std::ptr;
// use vk;
// use ::{util, VkcResult, Device};

// #[derive(Debug)]
// struct Inner {
//     handle: vks::core::VkAbstractTemplate,
//     device: Device,
// }

// #[derive(Debug, Clone)]
// pub struct AbstractTemplate {
//     inner: Arc<Inner>,
// }

// impl AbstractTemplate {
//     pub fn new() -> VkcResult<AbstractTemplate> {

//         let mut handle = 0;
//         unsafe {
//             ::check(device.vk().CreateAbstractTemplate(device.handle(), &create_info,
//                 ptr::null(), &mut handle));
//         }

//         Ok(AbstractTemplate {
//             inner: Arc::new(Inner {
//                 handle,
//                 device,
//             })
//         })
//     }

//     pub fn handle(&self) -> vks::core::VkAbstractTemplate {
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



pub fn check(code: i32) {
    if code != vks::core::VK_SUCCESS { panic!("Error code: {}", code); }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {

    }
}
