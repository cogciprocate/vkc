use std::sync::Arc;
use std::mem;
use std::ptr;
use vk;
use ::{VkcResult, Instance, Surface};
use queue::{self, Queue};

unsafe fn device_is_suitable(instance: &Instance, surface: &Surface, device: vk::PhysicalDevice,
        queue_flags: vk::QueueFlags) -> bool
{
    let mut device_properties: vk::PhysicalDeviceProperties = mem::uninitialized();
    let mut device_features: vk::PhysicalDeviceFeatures = mem::uninitialized();
    instance.vk().GetPhysicalDeviceProperties(device, &mut device_properties);
    instance.vk().GetPhysicalDeviceFeatures(device, &mut device_features);

    queue::queue_families(instance, surface, device, queue_flags).is_complete()
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


pub struct Inner {
    handle: vk::Device,
    physical_device: vk::PhysicalDevice,
    features: vk::PhysicalDeviceFeatures,
    vk: vk::DevicePointers,
    instance: Instance,
}

pub struct Device {
    inner: Arc<Inner>,
}

impl Device {
    pub fn new(instance: Instance, surface: &Surface, physical_device: vk::PhysicalDevice,
            queue_familiy_flags: vk::QueueFlags) -> Device
    {
        let queue_family_idx = queue::queue_families(&instance, surface,
            physical_device, queue_familiy_flags).family_idxs()[0];

        let queue_create_info = vk::DeviceQueueCreateInfo {
            sType: vk::STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            queueFamilyIndex: queue_family_idx as u32,
            queueCount: 1,
            pQueuePriorities: &1.0,
        };

        let features = device_features_none();

        let create_info = vk::DeviceCreateInfo {
            sType: vk::STRUCTURE_TYPE_DEVICE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            queueCreateInfoCount: 1,
            pQueueCreateInfos: &queue_create_info,
            enabledLayerCount: 0,
            ppEnabledLayerNames: ptr::null(),
            enabledExtensionCount: 0,
            ppEnabledExtensionNames: ptr::null(),
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
                features,
                vk,
                instance,
            }),
        }
    }

    #[inline]
    pub fn queue(&self, queue_family_index: u32, queue_index: u32) -> VkcResult<Queue> {
        Err(())
    }

    #[inline]
    pub fn vk(&self) -> &vk::DevicePointers {
        &self.inner.vk
    }

    #[inline]
    pub fn handle(&self) -> vk::Device {
        self.inner.handle
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