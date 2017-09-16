use std::os::raw::c_void;
use std::ptr;
use std::sync::atomic::AtomicBool;
use winit;
use vk;
use ::{Instance};

pub struct Surface {
    handle: vk::SurfaceKHR,
    instance: Instance,
    active: AtomicBool,
}

impl Surface {
    pub fn new(instance: Instance, window: &winit::Window) -> Surface {
        use winit::os::windows::WindowExt;
        let mut handle = 0;

        let create_info = vk::Win32SurfaceCreateInfoKHR {
            sType: vk::STRUCTURE_TYPE_WIN32_SURFACE_CREATE_INFO_KHR,
            pNext: ptr::null(),
            flags: 0,
            hinstance: ptr::null_mut(),
            hwnd: window.get_hwnd() as *mut c_void,
        };

        unsafe {
            ::check(instance.vk().CreateWin32SurfaceKHR(instance.handle(), &create_info, ptr::null(),
                &mut handle));
        }

        Surface {
            handle: handle,
            instance: instance,
            active: AtomicBool::new(false),
        }
    }

    pub fn handle(&self) -> vk::SurfaceKHR {
        self.handle
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            println!("Destroying surface...");
            self.instance.vk().DestroySurfaceKHR(self.instance.handle(), self.handle, ptr::null());
        }
    }
}