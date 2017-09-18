


use std::sync::Arc;
use std::ptr;
use vk;
use ::{util, VkcResult, Device};

#[derive(Debug)]
struct Inner {
    handle: vk::Semaphore,
    device: Device,
}

#[derive(Debug, Clone)]
pub struct Semaphore {
    inner: Arc<Inner>,
}

impl Semaphore {
    pub fn new(device: Device) -> VkcResult<Semaphore> {
        let create_info = vk::SemaphoreCreateInfo {
            sType: vk::STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
        };

        let mut handle = 0;
        unsafe {
            ::check(device.vk().CreateSemaphore(device.handle(), &create_info,
                ptr::null(), &mut handle));
        }

        Ok(Semaphore {
            inner: Arc::new(Inner {
                handle,
                device,
            })
        })
    }

    pub fn handle(&self) -> vk::Semaphore {
        self.inner.handle
    }

    pub fn device(&self) -> &Device {
        &self.inner.device
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().DestroySemaphore(self.device.handle(), self.handle, ptr::null());
        }
    }
}

