
use std::sync::Arc;
use std::ptr;
use vk;
use ::{util, VkcResult, Device};

#[derive(Debug)]
struct Inner {
    handle: vk::VkSampler,
    device: Device,
}

#[derive(Debug, Clone)]
pub struct Sampler {
    inner: Arc<Inner>,
}

impl Sampler {
    pub fn new(device: Device) -> VkcResult<Sampler> {
        let create_info = vk::VkSamplerCreateInfo {
            sType: vk::VK_STRUCTURE_TYPE_SAMPLER_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            magFilter: vk::VK_FILTER_LINEAR,
            minFilter: vk::VK_FILTER_LINEAR,
            mipmapMode: vk::VK_SAMPLER_MIPMAP_MODE_LINEAR,
            addressModeU: vk::VK_SAMPLER_ADDRESS_MODE_REPEAT,
            addressModeV: vk::VK_SAMPLER_ADDRESS_MODE_REPEAT,
            addressModeW: vk::VK_SAMPLER_ADDRESS_MODE_REPEAT,
            mipLodBias: 0.,
            // anisotropyEnable: vk::VK_FALSE,
            // maxAnisotropy: 1.,
            anisotropyEnable: vk::VK_TRUE,
            maxAnisotropy: 16.,
            compareEnable: vk::VK_FALSE,
            compareOp: vk::VK_COMPARE_OP_ALWAYS,
            minLod: 0.,
            maxLod: 0.,
            borderColor: vk::VK_BORDER_COLOR_INT_OPAQUE_BLACK,
            unnormalizedCoordinates: vk::VK_FALSE,
        };

        let mut handle = 0;
        unsafe {
            ::check(device.vk().vkCreateSampler(device.handle(), &create_info,
                ptr::null(), &mut handle));
        }

        Ok(Sampler {
            inner: Arc::new(Inner {
                handle,
                device,
            })
        })
    }

    pub fn handle(&self) -> vk::VkSampler {
        self.inner.handle
    }

    pub fn device(&self) -> &Device {
        &self.inner.device
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().vkDestroySampler(self.device.handle(), self.handle, ptr::null());
        }
    }
}