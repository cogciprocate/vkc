use std::mem;
use vk;
use instance::Instance;

unsafe fn device_is_suitable(vk: &vk::InstancePointers, device: vk::PhysicalDevice) -> bool {
    let mut device_properties: vk::PhysicalDeviceProperties = mem::uninitialized();
    let mut device_features: vk::PhysicalDeviceFeatures = mem::uninitialized();
    vk.GetPhysicalDeviceProperties(device, &mut device_properties);
    vk.GetPhysicalDeviceFeatures(device, &mut device_features);
    true
}

pub fn choose_device(instance: &Instance) -> vk::PhysicalDevice {
    let mut preferred_device = 0;

    for &device in instance.physical_devices() {
        if unsafe { device_is_suitable(&instance.vk, device) } {
            preferred_device = device;
            break;
        }
    }

    if preferred_device == 0 {
        panic!("Failed to find a suitable device.");
    } else {
        println!("Preferred device: {}", preferred_device);
    }

    preferred_device
}