
use std::sync::Arc;
use std::ptr;
use vk;
use vks;
use ::{util, VkcResult, Device, RenderPass, ImageView};


#[derive(Debug)]
struct Inner {
    handle: vk::VkFramebuffer,
    device: Device,
    render_pass: RenderPass,
    image_view: ImageView,
}

#[derive(Debug, Clone)]
pub struct Framebuffer {
    inner: Arc<Inner>,
}

impl Framebuffer {
    pub fn new(device: Device, render_pass: RenderPass, image_view: ImageView,
            swapchain_extent: vk::VkExtent2D) -> VkcResult<Framebuffer>
    {
        let attachments = [image_view.handle()];
        let create_info = vk::VkFramebufferCreateInfo {
            sType: vk::VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            renderPass: render_pass.handle(),
            attachmentCount: 1,
            pAttachments: attachments.as_ptr(),
            width: swapchain_extent.width,
            height: swapchain_extent.height,
            layers: 1,
        };

        let mut handle = 0;
        unsafe {
            ::check(device.vk().core.vkCreateFramebuffer(device.handle(), &create_info, ptr::null(),
                &mut handle));
        }

        Ok(Framebuffer {
            inner: Arc::new(Inner {
                handle,
                device,
                render_pass,
                image_view,
            })
        })
    }

    pub fn handle(&self) -> vk::VkFramebuffer {
        self.inner.handle
    }

    pub fn device(&self) -> &Device {
        &self.inner.device
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().core.vkDestroyFramebuffer(self.device.handle(), self.handle, ptr::null());
        }
    }
}


pub fn create_framebuffers(device: &Device, render_pass: &RenderPass,
        swapchain_image_views: &[ImageView], swapchain_extent: vk::VkExtent2D)
        -> VkcResult<Vec<Framebuffer>>
{
    swapchain_image_views.iter().map(|image_view| {
        Framebuffer::new(device.clone(), render_pass.clone(), image_view.clone(),
            swapchain_extent.clone())
    }).collect::<Result<Vec<_>, _>>()
}