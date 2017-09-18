
use std::sync::Arc;
use std::ptr;
use vk;
use ::{util, VkcResult, Device, Surface};


#[derive(Debug)]
struct Inner {
    handle: vk::CommandPool,
    device: Device,
}

#[derive(Debug, Clone)]
pub struct CommandPool {
    inner: Arc<Inner>,
}

impl CommandPool {
    pub fn new(device: Device, surface: &Surface, queue_family_flags: vk::QueueFlags)
        -> VkcResult<CommandPool>
    {
        let queue_family_idx = ::queue_families(device.instance(), surface,
            device.physical_device(), queue_family_flags).family_idxs()[0];

        let create_info = vk::CommandPoolCreateInfo {
            sType: vk::STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO,
            pNext: ptr::null(),
            // vk::COMMAND_POOL_CREATE_TRANSIENT_BIT
            // vk::COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT
            flags: 0,
            queueFamilyIndex: queue_family_idx as u32,
        };

        let mut handle = 0;
        unsafe {
            ::check(device.vk().CreateCommandPool(device.handle(), &create_info,
                ptr::null(), &mut handle));
        }

        Ok(CommandPool {
            inner: Arc::new(Inner {
                handle,
                device,
            })
        })
    }

    pub fn handle(&self) -> vk::CommandPool {
        self.inner.handle
    }

    pub fn device(&self) -> &Device {
        &self.inner.device
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().DestroyCommandPool(self.device.handle(), self.handle, ptr::null());
        }
    }
}