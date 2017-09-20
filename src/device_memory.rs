
use std::sync::Arc;
use std::ptr;
use vk;
use vks;
use ::{util, VkcResult, Device};

#[derive(Debug)]
struct Inner {
    handle: vk::VkDeviceMemory,
    device: Device,
}

#[derive(Debug, Clone)]
pub struct DeviceMemory {
    inner: Arc<Inner>,
}

impl DeviceMemory {
    pub fn new(device: Device, allocation_size: u64, memory_type_index: u32) -> VkcResult<DeviceMemory> {
        let create_info = vk::VkMemoryAllocateInfo {
            sType: vk::VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
            pNext: ptr::null(),
            allocationSize: allocation_size,
            memoryTypeIndex: memory_type_index,
        };

        let mut handle = 0;
        unsafe {
            ::check(device.vk().core.vkAllocateMemory(device.handle(), &create_info,
                ptr::null(), &mut handle));
        }

        Ok(DeviceMemory {
            inner: Arc::new(Inner {
                handle,
                device,
            })
        })
    }

    pub fn handle(&self) -> vk::VkDeviceMemory {
        self.inner.handle
    }

    pub fn device(&self) -> &Device {
        &self.inner.device
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().core.vkFreeMemory(self.device.handle(), self.handle, ptr::null());
        }
    }
}