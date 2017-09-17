use std::sync::Arc;
use std::ffi::CStr;
use std::ptr;
use std::path::Path;
use std::fs::File;
use std::io::{Read, BufReader};
use vk;
use ::Device;



struct Inner {
    handle: vk::ShaderModule,
    device: Device,
}

pub struct ShaderModule {
    inner: Arc<Inner>,
}

impl ShaderModule {
    pub fn new(device: Device, code: &[u8]) -> ShaderModule {
        let create_info = vk::ShaderModuleCreateInfo {
            sType: vk::STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            codeSize: code.len(),
            pCode: code.as_ptr() as *const u32,
        };

        let mut handle = 0;
        unsafe {
            ::check(device.vk().CreateShaderModule(device.handle(), &create_info,
                ptr::null(), &mut handle));
        }

        ShaderModule {
            inner: Arc::new(Inner {
                handle,
                device,
            })
        }
    }

    pub fn handle(&self) -> vk::ShaderModule {
        self.inner.handle
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().DestroyShaderModule(self.device.handle(), self.handle, ptr::null());
        }
    }
}

