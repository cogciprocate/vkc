use std::ptr;
use smallvec::SmallVec;
use vk;
use ::{VkcResult, Instance, Device, Surface};

pub struct QueueFamilyIndices {
    // family_idx: i32,
    physical_device: vk::PhysicalDevice,
    flags: vk::QueueFlags,
    flag_idxs: SmallVec<[i32; 64]>,
    presentation_support_idxs: SmallVec<[i32; 64]>,
}

impl QueueFamilyIndices {
    pub fn new(physical_device: vk::PhysicalDevice, flags: vk::QueueFlags) -> QueueFamilyIndices {
        QueueFamilyIndices {
            flag_idxs: SmallVec::new(),
            presentation_support_idxs: SmallVec::new(),
            physical_device,
            flags
        }
    }

    pub fn is_complete(&self) -> bool {
        // self.family_idx >= 0
        self.flag_idxs.len() > 0
    }

    pub fn family_idxs(&self) -> &[i32] {
        &self.flag_idxs

        // let mut i = 0i32;
        // for queue_family in &queue_families {
        //     if (queue_family.queueCount > 0) && (queue_family.queueFlags & queue_flags) != 0 {
        //         indices.family_idx = i;
        //     }
        //     if indices.is_complete() {
        //         break;
        //     }
        //     i += 1;
        // }
        // indices
    }
}

pub fn queue_families(instance: &Instance, surface: &Surface, device: vk::PhysicalDevice,
        queue_flags: vk::QueueFlags) -> QueueFamilyIndices
{
    let mut indices = QueueFamilyIndices::new(device, queue_flags);
    let mut queue_family_count = 0u32;
    let mut queue_families: Vec<vk::QueueFamilyProperties>;

    unsafe {
        instance.vk().GetPhysicalDeviceQueueFamilyProperties(device, &mut queue_family_count, ptr::null_mut());
        queue_families = Vec::with_capacity(queue_family_count as usize);
        queue_families.set_len(queue_family_count as usize);
        instance.vk().GetPhysicalDeviceQueueFamilyProperties(device, &mut queue_family_count, queue_families.as_mut_ptr());
    }

    let mut i = 0i32;
    for queue_family in &queue_families {
        if queue_family.queueCount > 0 && queue_family.queueFlags & queue_flags != 0 {
            indices.flag_idxs.push(i);
        }

        let mut presentation_support: vk::Bool32 = vk::FALSE;
        unsafe {
            ::check(instance.vk().GetPhysicalDeviceSurfaceSupportKHR(device, i as u32, surface.handle(),
                &mut presentation_support));
        }
        if queue_family.queueCount > 0 && presentation_support != 0 {
            indices.presentation_support_idxs.push(i);
        }

        if indices.is_complete() {
            break;
        }
        i += 1;
    }
    indices
}


pub struct Queue {
    handle: vk::Queue,
    device: Device,
    family_idx: u32,
    idx: u32,
}

impl Queue {
    // Queue families:
    // QUEUE_COMPUTE_BIT
    // QUEUE_FAMILY_IGNORED
    // QUEUE_GRAPHICS_BIT
    // QUEUE_SPARSE_BINDING_BIT
    // QUEUE_TRANSFER_BIT
    pub unsafe fn new(device: Device, queue_family_index: u32, queue_index: u32) -> VkcResult<Queue> {
        let mut handle = 0;
        device.vk().GetDeviceQueue(device.handle(), queue_family_index, queue_index, &mut handle);

        Ok(Queue {
            handle,
            device,
            family_idx: queue_family_index,
            idx: queue_index,
        })
    }
}