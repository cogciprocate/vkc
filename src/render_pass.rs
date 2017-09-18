use std::sync::Arc;
use std::ffi::CStr;
use std::ptr;
use vk;
use ::{util, VkcResult, Device, ShaderModule};


#[derive(Debug)]
struct Inner {
    handle: vk::RenderPass,
    device: Device,
}

#[derive(Debug, Clone)]
pub struct RenderPass {
    inner: Arc<Inner>,
}

impl RenderPass {
    pub fn new(device: Device, swap_chain_image_format: vk::Format) -> VkcResult<RenderPass> {
        let color_attachment = vk::AttachmentDescription {
            flags: 0,
            format: swap_chain_image_format,
            samples: vk::SAMPLE_COUNT_1_BIT,
            loadOp: vk::ATTACHMENT_LOAD_OP_CLEAR,
            storeOp: vk::ATTACHMENT_STORE_OP_STORE,
            stencilLoadOp: vk::ATTACHMENT_LOAD_OP_DONT_CARE,
            stencilStoreOp: vk::ATTACHMENT_STORE_OP_DONT_CARE,
            initialLayout: vk::IMAGE_LAYOUT_UNDEFINED,
            finalLayout: vk::IMAGE_LAYOUT_PRESENT_SRC_KHR,
        };

        let color_attachment_ref = vk::AttachmentReference {
            attachment: 0,
            layout: vk::IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL,
        };

        let subpass = vk::SubpassDescription {
            flags: 0,
            pipelineBindPoint: vk::PIPELINE_BIND_POINT_GRAPHICS,
            inputAttachmentCount: 0,
            pInputAttachments: ptr::null(),
            colorAttachmentCount: 1,
            pColorAttachments: &color_attachment_ref,
            pResolveAttachments: ptr::null(),
            pDepthStencilAttachment: ptr::null(),
            preserveAttachmentCount: 0,
            pPreserveAttachments: ptr::null(),
        };

        let dependency = vk::SubpassDependency {
            dependencyFlags: 0,
            srcSubpass: vk::SUBPASS_EXTERNAL,
            dstSubpass: 0,
            srcStageMask: vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
            srcAccessMask: 0,
            dstStageMask: vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
            dstAccessMask: vk::ACCESS_COLOR_ATTACHMENT_READ_BIT | vk::ACCESS_COLOR_ATTACHMENT_WRITE_BIT,
        };

        let render_pass_info = vk::RenderPassCreateInfo {
            sType: vk::STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            attachmentCount: 1,
            pAttachments: &color_attachment,
            subpassCount: 1,
            pSubpasses: &subpass,
            dependencyCount: 1,
            pDependencies: &dependency,
        };

        let mut handle = 0;
        unsafe {
            ::check(device.vk().CreateRenderPass(device.handle(), &render_pass_info, ptr::null(), &mut handle));
        }

        Ok(RenderPass {
            inner: Arc::new(Inner {
                handle,
                device,
            })
        })
    }

    pub fn handle(&self) -> vk::RenderPass {
        self.inner.handle
    }

    pub fn device(&self) -> &Device {
        &self.inner.device
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().DestroyRenderPass(self.device.handle(), self.handle, ptr::null());
        }
    }
}