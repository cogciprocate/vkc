

use std::sync::Arc;
use std::ptr;
use std::mem;
use vk;
use ::{util, VkcResult, Device, DeviceMemory};

#[derive(Debug)]
struct Inner {
    handle: vk::VkImage,
    device_memory: DeviceMemory,
    device: Device,
}

#[derive(Debug, Clone)]
pub struct Image {
    inner: Arc<Inner>,
}

impl Image {
    pub fn new(device: Device, extent: vk::VkExtent3D, format: vk::VkFormat,
            tiling: vk::VkImageTiling, usage: vk::VkImageUsageFlags,
            memory_properties: vk::VkMemoryPropertyFlags) -> VkcResult<Image>
    {
        let create_info = vk::VkImageCreateInfo {
            sType: vk::VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            imageType: vk::VK_IMAGE_TYPE_2D,
            // format: vk::VK_FORMAT_R8G8B8A8_UNORM,
            format,
            extent: extent,
            mipLevels: 1,
            arrayLayers: 1,
            samples: vk::VK_SAMPLE_COUNT_1_BIT,
            // tiling: vk::VK_IMAGE_TILING_OPTIMAL,
            tiling,
            // usage: vk::VK_IMAGE_USAGE_TRANSFER_DST_BIT | vk::VK_IMAGE_USAGE_SAMPLED_BIT,
            usage,
            sharingMode: vk::VK_SHARING_MODE_EXCLUSIVE,
            queueFamilyIndexCount: 0,
            pQueueFamilyIndices: ptr::null(),
            initialLayout: vk::VK_IMAGE_LAYOUT_UNDEFINED,
        };

        let mut handle = 0;
        unsafe {
            ::check(device.vk().vkCreateImage(device.handle(), &create_info,
                ptr::null(), &mut handle));
        }

        // Memory Requirements:
        let mut mem_requirements: vk::VkMemoryRequirements;
        unsafe {
            mem_requirements = mem::uninitialized();
            device.vk().core.vkGetImageMemoryRequirements(device.handle(), handle,
                &mut mem_requirements);
        }

        let memory_type_index = ::find_memory_type(&device, mem_requirements.memoryTypeBits,
            memory_properties);

        let alloc_info = vk::VkMemoryAllocateInfo {
            sType: vk::VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
            pNext: ptr::null(),
            allocationSize: mem_requirements.size,
            memoryTypeIndex: memory_type_index,
        };

        println!("Image: {:?}", mem_requirements);

        let device_memory = DeviceMemory::new(device.clone(), mem_requirements.size,
            memory_type_index)?;

        unsafe {
            ::check(device.vk().vkBindImageMemory(device.handle(), handle,
                device_memory.handle(), 0));
        }

        Ok(Image {
            inner: Arc::new(Inner {
                handle,
                device_memory,
                device,
            })
        })
    }

    pub fn handle(&self) -> vk::VkImage {
        self.inner.handle
    }

    pub fn device_memory(&self) -> &DeviceMemory {
        &self.inner.device_memory
    }

    pub fn device(&self) -> &Device {
        &self.inner.device
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().vkDestroyImage(self.device.handle(), self.handle, ptr::null());
        }
    }
}