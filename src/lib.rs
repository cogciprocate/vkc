//! vkc - Vulkan Compute

#![allow(unused_extern_crates, unused_imports, dead_code, unused_variables)]

extern crate libloading as lib;
extern crate smallvec;
// pub extern crate vk_sys as vk;
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
pub mod vulkan_h;
pub mod device;
pub mod util;

use std::ffi::OsStr;
use std::os::raw::c_void;
use std::mem;
use std::ptr;
use winit::{EventsLoop, WindowBuilder, Window, CreationError, ControlFlow, Event, WindowEvent};
use loader::Loader;
pub use vulkan_h as vk;
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

pub type VkcResult<T> = Result<T, ()>;


pub fn draw_frame(device: &Device, swapchain: &Swapchain, image_available_semaphore: &Semaphore,
        render_finished_semaphore: &Semaphore, command_buffers: &[vk::CommandBuffer])
        -> VkcResult<()>
{
    let mut image_index = 0u32;
    unsafe {
        check(device.vk().AcquireNextImageKHR(device.handle(), swapchain.handle(), u64::max_value(),
            image_available_semaphore.handle(), 0, &mut image_index));
    }

    let wait_semaphores = [image_available_semaphore.handle()];
    let wait_stages = [vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT];
    let signal_semaphores = [render_finished_semaphore.handle()];

    let submit_info = vk::SubmitInfo {
        sType: vk::STRUCTURE_TYPE_SUBMIT_INFO,
        pNext: ptr::null(),
        waitSemaphoreCount: wait_semaphores.len() as u32,
        pWaitSemaphores: wait_semaphores.as_ptr(),
        pWaitDstStageMask: wait_stages.as_ptr(),
        commandBufferCount: 1,
        pCommandBuffers: command_buffers.get(image_index as usize).unwrap(),
        signalSemaphoreCount: signal_semaphores.len() as u32,
        pSignalSemaphores: signal_semaphores.as_ptr(),
    };

    unsafe { check(device.vk().QueueSubmit(device.queue(0), 1, &submit_info, 0)); }

    let swapchains = [swapchain.handle()];

    let present_info = vk::PresentInfoKHR {
        sType: vk::STRUCTURE_TYPE_PRESENT_INFO_KHR,
        pNext: ptr::null(),
        waitSemaphoreCount: signal_semaphores.len() as u32,
        pWaitSemaphores: signal_semaphores.as_ptr(),
        swapchainCount: swapchains.len() as u32,
        pSwapchains: swapchains.as_ptr(),
        pImageIndices: &image_index,
        pResults: ptr::null_mut(),
    };

    unsafe {
        check(device.vk().QueuePresentKHR(device.queue(0), &present_info));
        check(device.vk().QueueWaitIdle(device.queue(0)));
    }

    Ok(())
}


///////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////// TEMPLATE /////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////
// use std::sync::Arc;
// use std::ptr;
// use vk;
// use ::{util, VkcResult, Device};

// #[derive(Debug)]
// struct Inner {
//     handle: vk::AbstractTemplate,
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



pub fn check(code: u32) {
    if code != vk::SUCCESS { panic!("Error code: {}", code); }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {

    }
}
