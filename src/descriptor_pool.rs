
use std::sync::Arc;
use std::ptr;
use vk;
use ::{util, VkcResult, Device};

#[derive(Debug)]
struct Inner {
    handle: vk::VkDescriptorPool,
    device: Device,
}

#[derive(Debug, Clone)]
pub struct DescriptorPool {
    inner: Arc<Inner>,
}

impl DescriptorPool {
    pub fn new(device: Device) -> VkcResult<DescriptorPool> {
        let pool_size = vk::VkDescriptorPoolSize {
            type_: vk::VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
            descriptorCount: 1,
        };

        let create_info = vk::VkDescriptorPoolCreateInfo {
            sType: vk::VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO,
            pNext: ptr::null(),
            // optional flag similar to command pools that determines if
            // individual descriptor sets can be freed or not:
            // `VK_DESCRIPTOR_POOL_CREATE_FREE_DESCRIPTOR_SET_BIT`:
            flags: 0,
            maxSets: 1,
            poolSizeCount: 1,
            pPoolSizes: &pool_size,
        };

        let mut handle = 0;
        unsafe {
            ::check(device.vk().vkCreateDescriptorPool(device.handle(), &create_info,
                ptr::null(), &mut handle));
        }

        Ok(DescriptorPool {
            inner: Arc::new(Inner {
                handle,
                device,
            })
        })
    }

    pub fn handle(&self) -> vk::VkDescriptorPool {
        self.inner.handle
    }

    pub fn device(&self) -> &Device {
        &self.inner.device
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().vkDestroyDescriptorPool(self.device.handle(), self.handle, ptr::null());
        }
    }
}