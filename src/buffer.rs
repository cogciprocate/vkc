

use std::sync::Arc;
use std::ptr;
use std::mem;
use vk;
use vks;
use ::{util, VkcResult, Device, Vertex};

#[derive(Debug)]
struct Inner {
    handle: vk::VkBuffer,
    device: Device,
}

#[derive(Debug, Clone)]
pub struct Buffer {
    inner: Arc<Inner>,
}

impl Buffer {
    pub fn new(device: Device, size: u64) -> VkcResult<Buffer> {
        let create_info = vk::VkBufferCreateInfo {
            sType: vk::VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            size: size,
            usage: vk::VK_BUFFER_USAGE_VERTEX_BUFFER_BIT,
            sharingMode: vk::VK_SHARING_MODE_EXCLUSIVE,
            queueFamilyIndexCount: 0,
            pQueueFamilyIndices: ptr::null(),
        };

        let mut handle = 0;
        unsafe {
            ::check(device.vk().core.vkCreateBuffer(device.handle(), &create_info,
                ptr::null(), &mut handle));
        }

        Ok(Buffer {
            inner: Arc::new(Inner {
                handle,
                device,
            })
        })
    }

    pub fn handle(&self) -> vk::VkBuffer {
        self.inner.handle
    }

    pub fn device(&self) -> &Device {
        &self.inner.device
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().core.vkDestroyBuffer(self.device.handle(), self.handle, ptr::null());
        }
    }
}