use std::sync::Arc;
use std::ffi::{self, CStr};
use std::ptr;
use std::mem;
use std::os::raw::{c_char, c_void};
use vk;
use ::{VkcResult, Loader};


static VALIDATION_LAYERS: [&[u8]; 1] = [
    b"VK_LAYER_LUNARG_standard_validation\0"
];

#[cfg(debug_assertions)]
const ENABLE_VALIDATION_LAYERS: bool = true;
#[cfg(not(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = false;

static REQUIRED_EXTENSIONS: [&[u8]; 2] = [
    b"VK_KHR_surface",
    b"VK_KHR_win32_surface",
];


extern "system" fn __debug_callback(_flags: vk::DebugReportFlagsEXT,
        _obj_type: vk::DebugReportObjectTypeEXT, _obj: u64, _location: usize, _code: i32,
        _layer_prefix: *const c_char, msg: *const c_char, _user_data: *mut c_void) -> u32
{
    unsafe {
        println!("{}", CStr::from_ptr(msg).to_str().unwrap());
    }
    vk::FALSE
}


// fn create_debug_report_callback_ext(instance: &Instance,
//         create_info: &vk::DebugReportCallbackCreateInfoEXT, allocator: vk::DebugReportCallbackEXT)
// {
//     let create_drcb = instance.get_instance_proc_addr(b"vkCreateDebugReportCallbackEXT".as_ptr() as *const i8);
// }

fn check_validation_layer_support(loader: &Loader, print: bool) -> bool {
    let mut layer_count = 0u32;
    let mut available_layers: Vec<vk::LayerProperties>;
    unsafe {
        ::check(loader.entry_points().EnumerateInstanceLayerProperties(&mut layer_count, ptr::null_mut()));
        available_layers = Vec::with_capacity(layer_count as usize);
        available_layers.set_len(layer_count as usize);
        ::check(loader.entry_points().EnumerateInstanceLayerProperties(&mut layer_count, available_layers.as_mut_ptr()));

        // Print available layers:
        if print {
            for layer_props in &available_layers {
                println!("Available layer: '{}'",
                    CStr::from_ptr(layer_props.layerName.as_ptr()).to_str().unwrap());
            }
        }

        // Verify that validation layer is available:
        for &layer_name in (&VALIDATION_LAYERS[..]).iter() {
            let mut layer_found = false;
            for layer_props in &available_layers {
                if CStr::from_ptr(layer_name.as_ptr() as *const c_char) ==
                    CStr::from_ptr(layer_props.layerName.as_ptr())
                {
                    if print { println!("Layer validated: '{}'",
                        CStr::from_ptr(layer_name.as_ptr() as *const c_char).to_str().unwrap()); }
                    layer_found = true;
                    break;
                }
            }
            if !layer_found { return false; }
        }
    }
    true
}

/// Currently returns all available extensions.
fn enumerate_instance_extension_properties(loader: &Loader) -> Vec<vk::ExtensionProperties> {
    let mut avail_ext_count = 0u32;
    let mut avail_exts: Vec<vk::ExtensionProperties>;
    unsafe {
        ::check(loader.entry_points().EnumerateInstanceExtensionProperties(ptr::null(),
            &mut avail_ext_count, ptr::null_mut()));

        avail_exts = Vec::with_capacity(avail_ext_count as usize);
        avail_exts.set_len(avail_ext_count as usize);
        ::check(loader.entry_points().EnumerateInstanceExtensionProperties(ptr::null(),
            &mut avail_ext_count, avail_exts.as_mut_ptr()));

        // Print available:
        for ext in &avail_exts {
            let name = (&ext.extensionName) as *const c_char;
            println!("Available instance extension: '{}' (version: {})",
                CStr::from_ptr(name).to_str().unwrap(), ext.specVersion);
        }
    }

    avail_exts
}

unsafe fn extension_names<'a>(extensions: &'a [vk::ExtensionProperties]) -> Vec<*const c_char> {
    extensions.iter().map(|ext| {
        let name = (&ext.extensionName) as *const c_char;
        println!("Enabling instance extension: '{}' (version: {})",
            CStr::from_ptr(name).to_str().unwrap(), ext.specVersion);
        name
        }).collect()
}

unsafe fn enumerate_physical_devices(instance: vk::Instance, vk: &vk::InstancePointers) -> Vec<vk::PhysicalDevice> {
    let mut device_count = 0;
    ::check(vk.EnumeratePhysicalDevices(instance, &mut device_count, ptr::null_mut()));
    if device_count == 0 { panic!("No physical devices found."); }
    let mut devices = Vec::with_capacity(device_count as usize);
    devices.set_len(device_count as usize);
    ::check(vk.EnumeratePhysicalDevices(instance, &mut device_count, devices.as_mut_ptr()));
    println!("Available devices: {:?}", devices);
    devices
}

pub fn enabled_layer_names(loader: &Loader, print: bool) -> Vec<*const c_char> {
    if ENABLE_VALIDATION_LAYERS && !check_validation_layer_support(loader, print) {
        panic!("Unable to enable validation layers.");
    }
    if ENABLE_VALIDATION_LAYERS {
         (&VALIDATION_LAYERS[..]).iter().map(|lyr_name|
            lyr_name.as_ptr() as *const c_char).collect()
    } else {
        Vec::new()
    }
}


#[derive(Debug)]
struct Inner {
    handle: vk::Instance,
    pub vk: vk::InstancePointers,
    loader: Loader,
    debug_callback: Option<vk::DebugReportCallbackEXT>,
    physical_devices: Vec<vk::PhysicalDevice>,
}

#[derive(Debug, Clone)]
pub struct Instance {
    inner: Arc<Inner>,
}

impl Instance {
    pub unsafe fn new(app_info: &vk::ApplicationInfo) -> VkcResult<Instance> {
        let loader = Loader::new()?;

        // Layers:
        let enabled_layer_names = enabled_layer_names(&loader, true);

        // Extensions:
        let extensions = enumerate_instance_extension_properties(&loader);
        let extension_names = extension_names(extensions.as_slice());

        // Instance:
        let info = vk::InstanceCreateInfo {
            sType: vk::STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            pApplicationInfo: app_info,
            enabledLayerCount: enabled_layer_names.len() as u32,
            ppEnabledLayerNames: enabled_layer_names.as_ptr(),
            enabledExtensionCount: extension_names.len() as u32,
            ppEnabledExtensionNames:extension_names.as_ptr(),
        };

        let mut handle = 0;
        ::check(loader.entry_points().CreateInstance(&info, ptr::null(), &mut handle));

        // Function pointers:
        let vk = vk::InstancePointers::load(|name|
            mem::transmute(loader.get_instance_proc_addr(handle, name.as_ptr())));

        let debug_callback = if ENABLE_VALIDATION_LAYERS {
            let create_info = vk::DebugReportCallbackCreateInfoEXT {
                sType:  vk::STRUCTURE_TYPE_DEBUG_REPORT_CALLBACK_CREATE_INFO_EXT,
                pNext: ptr::null(),
                flags: vk::DEBUG_REPORT_ERROR_BIT_EXT | vk::DEBUG_REPORT_WARNING_BIT_EXT,
                pfnCallback: __debug_callback,
                pUserData: ptr::null_mut(),
            };

            let mut callback: vk::DebugReportCallbackEXT = 0;
            if vk.CreateDebugReportCallbackEXT(handle, &create_info, ptr::null(), &mut callback) != vk::SUCCESS {
                panic!("failed to set up debug callback");
            } else {
                println!("Debug report callback initialized.");
            }
            Some(callback)
        } else {
            None
        };

        // Device:
        let physical_devices = enumerate_physical_devices(handle, &vk);

        Ok(Instance {
            inner: Arc::new(Inner {
                handle,
                vk,
                loader,
                debug_callback,
                physical_devices,
            }),
        })
    }

    #[inline]
    pub fn vk(&self) -> &vk::InstancePointers {
        &self.inner.vk
    }

    #[inline]
    pub fn handle(&self) -> vk::Instance {
        self.inner.handle
    }

    #[inline]
    pub fn get_instance_proc_addr(&self, name: *const i8) -> extern "system" fn() -> () {
        self.inner.loader.get_instance_proc_addr(self.inner.handle, name)
    }

    #[inline]
    pub fn physical_devices(&self) -> &[vk::PhysicalDevice] {
        self.inner.physical_devices.as_slice()
    }

    #[inline]
    pub fn loader(&self) -> &Loader {
        &self.inner.loader
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        unsafe {
            println!("Destroying debug callback...");
            if let Some(callback) = self.debug_callback {
                self.vk.DestroyDebugReportCallbackEXT(self.handle, callback, ptr::null());
            }

            println!("Destroying instance...");
            self.vk.DestroyInstance(self.handle, ptr::null());
        }
    }
}