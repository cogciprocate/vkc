
use std::sync::Arc;
use std::ptr;
use std::mem;
use vk;
use vks;
use ::{util, VkcResult, Device, DeviceMemory};


fn find_memory_type(device: &Device, type_filter: u32, properties: vk::VkMemoryPropertyFlags) -> u32 {
    let mut mem_properties: vk::VkPhysicalDeviceMemoryProperties;
    unsafe {
        mem_properties = mem::uninitialized();
        device.instance().vk().core.vkGetPhysicalDeviceMemoryProperties(device.physical_device(),
            &mut mem_properties);
    }

    for i in 0..mem_properties.memoryTypeCount {
        if (type_filter & (1 << i)) != 0 &&
            (mem_properties.memoryTypes[i as usize].propertyFlags & properties) == properties
        {
            return i;
        }
    }
    panic!("Failed to find suitable memory type.");
}



#[derive(Debug)]
struct Inner {
    handle: vk::VkBuffer,
    device_memory: DeviceMemory,
    device: Device,
}

#[derive(Debug, Clone)]
pub struct Buffer {
    inner: Arc<Inner>,
}

impl Buffer {
    pub fn new(device: Device, size: u64, usage: vk::VkBufferUsageFlags,
            sharing_mode: vk::VkSharingMode, properties: vk::VkMemoryPropertyFlags)
            -> VkcResult<Buffer>
    {
        let create_info = vk::VkBufferCreateInfo {
            sType: vk::VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            size: size,
            usage: usage,
            sharingMode: sharing_mode,
            queueFamilyIndexCount: 0,
            pQueueFamilyIndices: ptr::null(),
        };

        let mut handle = 0;
        unsafe {
            ::check(device.vk().core.vkCreateBuffer(device.handle(), &create_info,
                ptr::null(), &mut handle));
        }

        let mut mem_requirements: vk::VkMemoryRequirements;
        unsafe {
            mem_requirements = mem::uninitialized();
            device.vk().core.vkGetBufferMemoryRequirements(device.handle(), handle,
                &mut mem_requirements);
        }

        // * Use a memory heap that is host coherent, indicated with
        //   VK_MEMORY_PROPERTY_HOST_COHERENT_BIT (or)
        // * Call vkFlushMappedMemoryRanges to after writing to the mapped
        //   memory, and call vkInvalidateMappedMemoryRanges before reading from
        //   the mapped memory
        let memory_type_index = find_memory_type(&device, mem_requirements.memoryTypeBits,
            properties);

        let alloc_info = vk::VkMemoryAllocateInfo {
            sType: vk::VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
            pNext: ptr::null(),
            allocationSize: mem_requirements.size,
            memoryTypeIndex: memory_type_index,
        };

        let device_memory = DeviceMemory::new(device.clone(), mem_requirements.size,
            memory_type_index)?;

        unsafe {
            ::check(device.vk().core.vkBindBufferMemory(device.handle(), handle,
                device_memory.handle(), 0));
        }

        Ok(Buffer {
            inner: Arc::new(Inner {
                handle,
                device,
                device_memory,
            })
        })
    }

    pub fn handle(&self) -> vk::VkBuffer {
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
            self.device.vk().core.vkDestroyBuffer(self.device.handle(), self.handle, ptr::null());
        }
    }
}