use std::sync::Arc;
use std::mem;
use std::ptr;
use std::ffi::CStr;
use std::os::raw::c_char;
use vk;
use ::{VkcResult, Instance, Surface, SwapchainSupportDetails};
use queue::{self, Queue};
use instance;


static REQUIRED_EXTENSIONS: [&[u8]; 1] = [
    b"VK_KHR_swapchain\0",
];

// bool checkDeviceExtensionSupport(VkPhysicalDevice device) {
//     uint32_t extensionCount;
//     vkEnumerateDeviceExtensionProperties(device, nullptr, &extensionCount, nullptr);

//     std::vector<VkExtensionProperties> availableExtensions(extensionCount);
//     vkEnumerateDeviceExtensionProperties(device, nullptr, &extensionCount, availableExtensions.data());

//     std::set<std::string> requiredExtensions(deviceExtensions.begin(), deviceExtensions.end());

//     for (const auto& extension : availableExtensions) {
//         requiredExtensions.erase(extension.extensionName);
//     }

//     return requiredExtensions.empty();
// }

fn check_device_extension_support(instance: &Instance, device: vk::PhysicalDevice) -> bool {
    let mut avail_ext_count = 0u32;
    let mut avail_exts: Vec<vk::ExtensionProperties>;
    unsafe {
        ::check(instance.vk().EnumerateDeviceExtensionProperties(device, ptr::null(),
            &mut avail_ext_count, ptr::null_mut()));
        avail_exts = Vec::with_capacity(avail_ext_count as usize);
        avail_exts.set_len(avail_ext_count as usize);
        ::check(instance.vk().EnumerateDeviceExtensionProperties(device, ptr::null(),
            &mut avail_ext_count, avail_exts.as_mut_ptr()));

        // Print available:
        for ext in &avail_exts {
                let name = (&ext.extensionName) as *const c_char;
                println!("Available device extension: '{}' (version: {})",
                    CStr::from_ptr(name).to_str().unwrap(), ext.specVersion);
        };

        for reqd_ext_name in &REQUIRED_EXTENSIONS[..] {
            let mut ext_avail = false;
            for avail_ext in &avail_exts {
                if CStr::from_ptr(reqd_ext_name.as_ptr() as *const c_char) ==
                    CStr::from_ptr(avail_ext.extensionName.as_ptr())
                {
                    println!("Required device extension available: '{}'",
                        CStr::from_ptr(reqd_ext_name.as_ptr() as *const c_char).to_str().unwrap());
                    ext_avail = true;
                    break;
                }
            }
            if !ext_avail { return false; }
        }
    }
    true
}

unsafe fn device_is_suitable(instance: &Instance, surface: &Surface, device: vk::PhysicalDevice,
        queue_flags: vk::QueueFlags) -> bool
{
    let mut device_properties: vk::PhysicalDeviceProperties = mem::uninitialized();
    let mut device_features: vk::PhysicalDeviceFeatures = mem::uninitialized();
    instance.vk().GetPhysicalDeviceProperties(device, &mut device_properties);
    instance.vk().GetPhysicalDeviceFeatures(device, &mut device_features);

    let extensions_supported = check_device_extension_support(instance, device);

    let mut swap_chain_adequate = false;
    if extensions_supported {
        let swap_chain_details = SwapchainSupportDetails::new(instance, surface, device);
        swap_chain_adequate = !swap_chain_details.formats.is_empty() &&
            !swap_chain_details.present_modes.is_empty()
    }

    queue::queue_families(instance, surface, device, queue_flags).is_complete() &&
        extensions_supported &&
        swap_chain_adequate
}

pub fn choose_physical_device(instance: &Instance, surface: &Surface, queue_flags: vk::QueueFlags)
        -> vk::PhysicalDevice
{
    let mut preferred_device = 0;

    for &device in instance.physical_devices() {
        if unsafe { device_is_suitable(instance, surface, device, queue_flags) } {
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

fn device_features_none() -> vk::PhysicalDeviceFeatures {
    vk::PhysicalDeviceFeatures {
        robustBufferAccess: vk::FALSE,
        fullDrawIndexUint32: vk::FALSE,
        imageCubeArray: vk::FALSE,
        independentBlend: vk::FALSE,
        geometryShader: vk::FALSE,
        tessellationShader: vk::FALSE,
        sampleRateShading: vk::FALSE,
        dualSrcBlend: vk::FALSE,
        logicOp: vk::FALSE,
        multiDrawIndirect: vk::FALSE,
        drawIndirectFirstInstance: vk::FALSE,
        depthClamp: vk::FALSE,
        depthBiasClamp: vk::FALSE,
        fillModeNonSolid: vk::FALSE,
        depthBounds: vk::FALSE,
        wideLines: vk::FALSE,
        largePoints: vk::FALSE,
        alphaToOne: vk::FALSE,
        multiViewport: vk::FALSE,
        samplerAnisotropy: vk::FALSE,
        textureCompressionETC2: vk::FALSE,
        textureCompressionASTC_LDR: vk::FALSE,
        textureCompressionBC: vk::FALSE,
        occlusionQueryPrecise: vk::FALSE,
        pipelineStatisticsQuery: vk::FALSE,
        vertexPipelineStoresAndAtomics: vk::FALSE,
        fragmentStoresAndAtomics: vk::FALSE,
        shaderTessellationAndGeometryPointSize: vk::FALSE,
        shaderImageGatherExtended: vk::FALSE,
        shaderStorageImageExtendedFormats: vk::FALSE,
        shaderStorageImageMultisample: vk::FALSE,
        shaderStorageImageReadWithoutFormat: vk::FALSE,
        shaderStorageImageWriteWithoutFormat: vk::FALSE,
        shaderUniformBufferArrayDynamicIndexing: vk::FALSE,
        shaderSampledImageArrayDynamicIndexing: vk::FALSE,
        shaderStorageBufferArrayDynamicIndexing: vk::FALSE,
        shaderStorageImageArrayDynamicIndexing: vk::FALSE,
        shaderClipDistance: vk::FALSE,
        shaderCullDistance: vk::FALSE,
        shaderf3264: vk::FALSE,
        shaderInt64: vk::FALSE,
        shaderInt16: vk::FALSE,
        shaderResourceResidency: vk::FALSE,
        shaderResourceMinLod: vk::FALSE,
        sparseBinding: vk::FALSE,
        sparseResidencyBuffer: vk::FALSE,
        sparseResidencyImage2D: vk::FALSE,
        sparseResidencyImage3D: vk::FALSE,
        sparseResidency2Samples: vk::FALSE,
        sparseResidency4Samples: vk::FALSE,
        sparseResidency8Samples: vk::FALSE,
        sparseResidency16Samples: vk::FALSE,
        sparseResidencyAliased: vk::FALSE,
        variableMultisampleRate: vk::FALSE,
        inheritedQueries: vk::FALSE,
    }
}


#[derive(Debug)]
struct Inner {
    handle: vk::Device,
    physical_device: vk::PhysicalDevice,
    // features: vk::PhysicalDeviceFeatures,
    // queues: SmallVec<[u32; 32]>,
    queue_family_idx: u32,
    vk: vk::DevicePointers,
    instance: Instance,
}

#[derive(Debug, Clone)]
pub struct Device {
    inner: Arc<Inner>,
}

impl Device {
    pub fn new(instance: Instance, surface: &Surface, physical_device: vk::PhysicalDevice,
            queue_familiy_flags: vk::QueueFlags) -> Device
    {
        let queue_family_idx = queue::queue_families(&instance, surface,
            physical_device, queue_familiy_flags).family_idxs()[0] as u32;

        let queue_create_info = vk::DeviceQueueCreateInfo {
            sType: vk::STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            queueFamilyIndex: queue_family_idx,
            queueCount: 1,
            pQueuePriorities: &1.0,
        };

        let features = device_features_none();

        // createInfo.enabledExtensionCount = static_cast<uint32_t>(deviceExtensions.size());
        // createInfo.ppEnabledExtensionNames = deviceExtensions.data();

        let enabled_layer_names = instance::enabled_layer_names(instance.loader(), false);

        let enabled_extension_names: Vec<_> = (&REQUIRED_EXTENSIONS[..]).iter().map(|ext_name|
            ext_name.as_ptr() as *const c_char).collect();

        let create_info = vk::DeviceCreateInfo {
            sType: vk::STRUCTURE_TYPE_DEVICE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            queueCreateInfoCount: 1,
            pQueueCreateInfos: &queue_create_info,
            enabledLayerCount: enabled_layer_names.len() as u32,
            ppEnabledLayerNames: enabled_layer_names.as_ptr(),
            enabledExtensionCount: enabled_extension_names.len() as u32,
            ppEnabledExtensionNames: enabled_extension_names.as_ptr(),
            pEnabledFeatures: &features,
        };

        // Device:
        let mut handle = 0;
        unsafe {
            ::check(instance.vk().CreateDevice(physical_device, &create_info, ptr::null(), &mut handle));
        }

        // Function pointers:
        let vk = vk::DevicePointers::load(|name|
            unsafe { mem::transmute(instance.get_instance_proc_addr(name.as_ptr())) });

        Device {
            inner: Arc::new(Inner {
                handle,
                physical_device,
                // features,
                queue_family_idx,
                vk,
                instance,
            }),
        }
    }

    #[inline]
    pub fn queue(&self, queue_idx: u32) -> vk::Queue {
        let mut queue_handle = 0;
        unsafe {
            self.vk().GetDeviceQueue(self.inner.handle, self.inner.queue_family_idx, queue_idx,
                &mut queue_handle);
        }
        queue_handle
    }

    #[inline]
    pub fn vk(&self) -> &vk::DevicePointers {
        &self.inner.vk
    }

    #[inline]
    pub fn handle(&self) -> vk::Device {
        self.inner.handle
    }

    #[inline]
    pub fn physical_device(&self) -> vk::PhysicalDevice {
        self.inner.physical_device
    }

    #[inline]
    pub fn instance(&self) -> &Instance {
        &self.inner.instance
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        println!("Destroying device...");
        unsafe {
            self.vk.DestroyDevice(self.handle, ptr::null());
        }
    }
}