use std::sync::Arc;
use std::ffi::CStr;
use std::ptr;
use vk;
use vks;
use ::{util, VkcResult, Device, ShaderModule};


#[derive(Debug)]
struct Inner {
    handle: vk::VkRenderPass,
    device: Device,
}

#[derive(Debug, Clone)]
pub struct RenderPass {
    inner: Arc<Inner>,
}

impl RenderPass {
    pub fn new(device: Device, swapchain_image_format: vk::VkFormat,
            depth_image_format: vk::VkFormat) -> VkcResult<RenderPass>
    {
        let color_attachment = vk::VkAttachmentDescription {
            flags: 0,
            format: swapchain_image_format,
            samples: vk::VK_SAMPLE_COUNT_1_BIT,
            loadOp: vk::VK_ATTACHMENT_LOAD_OP_CLEAR,
            storeOp: vk::VK_ATTACHMENT_STORE_OP_STORE,
            stencilLoadOp: vk::VK_ATTACHMENT_LOAD_OP_DONT_CARE,
            stencilStoreOp: vk::VK_ATTACHMENT_STORE_OP_DONT_CARE,
            initialLayout: vk::VK_IMAGE_LAYOUT_UNDEFINED,
            finalLayout: vk::VK_IMAGE_LAYOUT_PRESENT_SRC_KHR,
        };

        let depth_attachment = vk::VkAttachmentDescription {
            flags: 0,
            format: depth_image_format,
            samples: vk::VK_SAMPLE_COUNT_1_BIT,
            loadOp: vk::VK_ATTACHMENT_LOAD_OP_CLEAR,
            storeOp: vk::VK_ATTACHMENT_STORE_OP_DONT_CARE,
            stencilLoadOp: vk::VK_ATTACHMENT_LOAD_OP_DONT_CARE,
            stencilStoreOp: vk::VK_ATTACHMENT_STORE_OP_DONT_CARE,
            initialLayout: vk::VK_IMAGE_LAYOUT_UNDEFINED,
            finalLayout: vk::VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };

        let color_attachment_ref = vk::VkAttachmentReference {
            attachment: 0,
            layout: vk::VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL,
        };

        let depth_attachment_ref = vk::VkAttachmentReference {
            attachment: 1,
            layout: vk::VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };

        let subpass = vk::VkSubpassDescription {
            flags: 0,
            pipelineBindPoint: vk::VK_PIPELINE_BIND_POINT_GRAPHICS,
            inputAttachmentCount: 0,
            pInputAttachments: ptr::null(),
            colorAttachmentCount: 1,
            pColorAttachments: &color_attachment_ref,
            pResolveAttachments: ptr::null(),
            pDepthStencilAttachment: &depth_attachment_ref,
            preserveAttachmentCount: 0,
            pPreserveAttachments: ptr::null(),
        };

        let dependency = vk::VkSubpassDependency {
            dependencyFlags: 0,
            srcSubpass: vk::VK_SUBPASS_EXTERNAL,
            dstSubpass: 0,
            srcStageMask: vk::VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
            srcAccessMask: 0,
            dstStageMask: vk::VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
            dstAccessMask: vk::VK_ACCESS_COLOR_ATTACHMENT_READ_BIT | vk::VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT,
        };

        let attachments = [color_attachment, depth_attachment];

        let create_info = vk::VkRenderPassCreateInfo {
            sType: vk::VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            attachmentCount: attachments.len() as u32,
            pAttachments: attachments.as_ptr(),
            subpassCount: 1,
            pSubpasses: &subpass,
            dependencyCount: 1,
            pDependencies: &dependency,
        };

        let mut handle = 0;
        unsafe {
            ::check(device.vk().core.vkCreateRenderPass(device.handle(), &create_info, ptr::null(), &mut handle));
        }

        Ok(RenderPass {
            inner: Arc::new(Inner {
                handle,
                device,
            })
        })
    }

    pub fn handle(&self) -> vk::VkRenderPass {
        self.inner.handle
    }

    pub fn device(&self) -> &Device {
        &self.inner.device
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().core.vkDestroyRenderPass(self.device.handle(), self.handle, ptr::null());
        }
    }
}