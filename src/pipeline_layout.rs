use std::sync::Arc;
use std::ffi::CStr;
use std::ptr;
use vk;
use ::{util, VkcResult, Device, ShaderModule};


#[derive(Debug)]
struct Inner {
    handle: vk::PipelineLayout,
    device: Device,
}

#[derive(Debug, Clone)]
pub struct PipelineLayout {
    inner: Arc<Inner>,
}

impl PipelineLayout {
    pub fn new(device: Device) -> VkcResult<PipelineLayout> {
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo {
            sType: vk::STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            setLayoutCount: 0,
            pSetLayouts: ptr::null(),
            pushConstantRangeCount: 0,
            pPushConstantRanges: ptr::null(),
        };

        let mut handle = 0;
        unsafe {
            ::check(device.vk().CreatePipelineLayout(device.handle(),
                &pipeline_layout_info, ptr::null(), &mut handle));
        }

        Ok(PipelineLayout {
            inner: Arc::new(Inner {
                handle,
                device,
            })
        })
    }

    pub fn handle(&self) -> vk::PipelineLayout {
        self.inner.handle
    }

    pub fn device(&self) -> &Device {
        &self.inner.device
    }
}


impl Drop for Inner {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().DestroyPipelineLayout(self.device.handle(), self.handle, ptr::null());
        }
    }
}