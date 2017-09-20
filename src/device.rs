use std::sync::Arc;
use std::mem;
use std::ptr;
use std::ffi::CStr;
use libc::c_char;
use vk;
use vks;
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

fn check_device_extension_support(instance: &Instance, device: vk::VkPhysicalDevice) -> bool {
    let mut avail_ext_count = 0u32;
    let mut avail_exts: Vec<vk::VkExtensionProperties>;
    unsafe {
        ::check(instance.vk().core.vkEnumerateDeviceExtensionProperties(device, ptr::null(),
            &mut avail_ext_count, ptr::null_mut()));
        avail_exts = Vec::with_capacity(avail_ext_count as usize);
        avail_exts.set_len(avail_ext_count as usize);
        ::check(instance.vk().core.vkEnumerateDeviceExtensionProperties(device, ptr::null(),
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

unsafe fn device_is_suitable(instance: &Instance, surface: &Surface, device: vk::VkPhysicalDevice,
        queue_flags: vk::VkQueueFlags) -> bool
{
    let mut device_properties: vk::VkPhysicalDeviceProperties = mem::uninitialized();
    let mut device_features: vk::VkPhysicalDeviceFeatures = mem::uninitialized();
    instance.vk().core.vkGetPhysicalDeviceProperties(device, &mut device_properties);
    instance.vk().core.vkGetPhysicalDeviceFeatures(device, &mut device_features);

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

pub fn choose_physical_device(instance: &Instance, surface: &Surface, queue_flags: vk::VkQueueFlags)
        -> VkcResult<vk::VkPhysicalDevice>
{
    let mut preferred_device = ptr::null_mut();

    for &device in instance.physical_devices() {
        if unsafe { device_is_suitable(instance, surface, device, queue_flags) } {
            preferred_device = device;
            break;
        }
    }

    if preferred_device.is_null() {
        panic!("Failed to find a suitable device.");
    } else {
        println!("Preferred device: {:?}", preferred_device);
    }

    Ok(preferred_device)
}

fn device_features_none() -> vk::VkPhysicalDeviceFeatures {
    vk::VkPhysicalDeviceFeatures {
        robustBufferAccess: vk::VK_FALSE,
        fullDrawIndexUint32: vk::VK_FALSE,
        imageCubeArray: vk::VK_FALSE,
        independentBlend: vk::VK_FALSE,
        geometryShader: vk::VK_FALSE,
        tessellationShader: vk::VK_FALSE,
        sampleRateShading: vk::VK_FALSE,
        dualSrcBlend: vk::VK_FALSE,
        logicOp: vk::VK_FALSE,
        multiDrawIndirect: vk::VK_FALSE,
        drawIndirectFirstInstance: vk::VK_FALSE,
        depthClamp: vk::VK_FALSE,
        depthBiasClamp: vk::VK_FALSE,
        fillModeNonSolid: vk::VK_FALSE,
        depthBounds: vk::VK_FALSE,
        wideLines: vk::VK_FALSE,
        largePoints: vk::VK_FALSE,
        alphaToOne: vk::VK_FALSE,
        multiViewport: vk::VK_FALSE,
        samplerAnisotropy: vk::VK_FALSE,
        textureCompressionETC2: vk::VK_FALSE,
        textureCompressionASTC_LDR: vk::VK_FALSE,
        textureCompressionBC: vk::VK_FALSE,
        occlusionQueryPrecise: vk::VK_FALSE,
        pipelineStatisticsQuery: vk::VK_FALSE,
        vertexPipelineStoresAndAtomics: vk::VK_FALSE,
        fragmentStoresAndAtomics: vk::VK_FALSE,
        shaderTessellationAndGeometryPointSize: vk::VK_FALSE,
        shaderImageGatherExtended: vk::VK_FALSE,
        shaderStorageImageExtendedFormats: vk::VK_FALSE,
        shaderStorageImageMultisample: vk::VK_FALSE,
        shaderStorageImageReadWithoutFormat: vk::VK_FALSE,
        shaderStorageImageWriteWithoutFormat: vk::VK_FALSE,
        shaderUniformBufferArrayDynamicIndexing: vk::VK_FALSE,
        shaderSampledImageArrayDynamicIndexing: vk::VK_FALSE,
        shaderStorageBufferArrayDynamicIndexing: vk::VK_FALSE,
        shaderStorageImageArrayDynamicIndexing: vk::VK_FALSE,
        shaderClipDistance: vk::VK_FALSE,
        shaderCullDistance: vk::VK_FALSE,
        shaderFloat64: vk::VK_FALSE,
        shaderInt64: vk::VK_FALSE,
        shaderInt16: vk::VK_FALSE,
        shaderResourceResidency: vk::VK_FALSE,
        shaderResourceMinLod: vk::VK_FALSE,
        sparseBinding: vk::VK_FALSE,
        sparseResidencyBuffer: vk::VK_FALSE,
        sparseResidencyImage2D: vk::VK_FALSE,
        sparseResidencyImage3D: vk::VK_FALSE,
        sparseResidency2Samples: vk::VK_FALSE,
        sparseResidency4Samples: vk::VK_FALSE,
        sparseResidency8Samples: vk::VK_FALSE,
        sparseResidency16Samples: vk::VK_FALSE,
        sparseResidencyAliased: vk::VK_FALSE,
        variableMultisampleRate: vk::VK_FALSE,
        inheritedQueries: vk::VK_FALSE,
    }
}


#[derive(Debug)]
struct Inner {
    handle: vk::VkDevice,
    physical_device: vk::VkPhysicalDevice,
    // features: vk::VkPhysicalDeviceFeatures,
    // queues: SmallVec<[u32; 32]>,
    queue_family_idx: u32,
    // vk: vk::VkDevicePointers,
    instance: Instance,
    loader: vks::DeviceProcAddrLoader,
}

#[derive(Debug, Clone)]
pub struct Device {
    inner: Arc<Inner>,
}

impl Device {
    pub fn new(instance: Instance, surface: &Surface, physical_device: vk::VkPhysicalDevice,
            queue_familiy_flags: vk::VkQueueFlags) -> VkcResult<Device>
    {
        let queue_family_idx = queue::queue_families(&instance, surface,
            physical_device, queue_familiy_flags).family_idxs()[0] as u32;

        let queue_create_info = vk::VkDeviceQueueCreateInfo {
            sType: vk::VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
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

        let create_info = vk::VkDeviceCreateInfo {
            sType: vk::VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO,
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
        let mut handle = ptr::null_mut();
        unsafe {
            ::check(instance.vk().core.vkCreateDevice(physical_device, &create_info, ptr::null(), &mut handle));
        }

        let mut loader = vks::DeviceProcAddrLoader::from_get_device_proc_addr(instance.vk().core.pfn_vkGetDeviceProcAddr);

        unsafe {
            loader.load_core(handle);
            // create_info.enabled_extensions.load_device(&mut loader, handle);
            // instance.loader().get_enabled_extensions().load_device(&mut loader, handle);
            // loader.load_khr_sampler_mirror_clamp_to_edge(handle);
            // loader.load_khr_draw_parameters(handle);
            loader.load_khr_swapchain(handle);
            // loader.load_khr_maintenance1(handle);
            // loader.load_amd_rasterization_order(handle);
            // loader.load_amd_draw_indirect_count(handle);
            // loader.load_amd_shader_ballot(handle);
            // loader.load_amd_shader_trinary_minmax(handle);
            // loader.load_amd_shader_explicit_vertex_parameter(handle);
            // loader.load_amd_gcn_shader(handle);
            // loader.load_amd_draw_indirect_count(handle);
            // loader.load_amd_negative_viewport_height(handle);
            // loader.load_amd_shader_info(handle);
            // loader.load_amd_wave_limits(handle);
            // loader.load_amd_texture_gather_bias_lod(handle);
            // loader.load_amd_programmable_sample_locations(handle);
            // loader.load_amd_mixed_attachment_samples(handle);
            // loader.load_ext_shader_subgroup_vote(handle);
            // loader.load_amd_gpa_interface(handle);
            // loader.load_ext_shader_subgroup_ballot(handle);
        }


        Ok(Device {
            inner: Arc::new(Inner {
                handle,
                physical_device,
                // features,
                queue_family_idx,
                instance,
                loader,
            }),
        })
    }

    #[inline]
    pub fn queue(&self, queue_idx: u32) -> vk::VkQueue {
        let mut queue_handle = ptr::null_mut();
        unsafe {
            self.vk().core.vkGetDeviceQueue(self.inner.handle, self.inner.queue_family_idx, queue_idx,
                &mut queue_handle);
        }
        queue_handle
    }

    #[inline]
    pub fn vk(&self) -> &vks::DeviceProcAddrLoader {
        // &self.inner.vk
        &self.inner.loader
    }

    #[inline]
    pub fn handle(&self) -> vk::VkDevice {
        self.inner.handle
    }

    #[inline]
    pub fn physical_device(&self) -> vk::VkPhysicalDevice {
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
            self.instance.vk().core.vkDestroyDevice(self.handle, ptr::null());
        }
    }
}