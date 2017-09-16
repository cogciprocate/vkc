//! vkc - Vulkan Compute

#![allow(unused_extern_crates, unused_imports, dead_code, unused_variables)]

extern crate vk_sys as vk;
extern crate winit;
extern crate libloading as lib;

mod version;
mod loader;
mod instance;
mod device;

use std::ffi::OsStr;
use std::mem;
use std::ptr;
use winit::{EventsLoop, WindowBuilder, Window, CreationError, ControlFlow, Event, WindowEvent};
use version::Version;
use loader::Loader;
use instance::Instance;

fn check(code: u32) {
    if code != vk::SUCCESS { panic!("Error code: {}", code); }
}

struct App {
    // loader: Loader,
    instance: Instance,
}

impl Drop for App {
    fn drop(&mut self) {
        // unsafe { self.instance.vk.DestroyInstance(self.instance.instance, ptr::null()); }
    }
}

/// Main function.
pub unsafe fn run() {
    let (window, events_loop) = init_window();
    let mut app = init();
    main_loop(window, events_loop, &mut app);
}

fn init_window() -> (Window, EventsLoop) {
    let events_loop = EventsLoop::new();
    let builder = WindowBuilder::new();
    let window = builder.build(&events_loop).unwrap();
    (window, events_loop)
}

unsafe fn init() -> App {
    let app_name = b"Hello Triangle\0";
    let engine_name = b"No Engine\0";

    let app_info = vk::ApplicationInfo {
        sType: vk::STRUCTURE_TYPE_APPLICATION_INFO,
        pNext: ptr::null(),
        pApplicationName: app_name.as_ptr() as *const i8,
        applicationVersion: Version::new(1, 0, 0).into(),
        pEngineName: engine_name.as_ptr() as *const i8,
        engineVersion: Version::new(1, 0, 0).into(),
        apiVersion: Version::new(1, 0, 0).into(),
    };

    let instance = Instance::new(&app_info);
    let device = device::choose_device(&instance);

    App { instance }
}

unsafe fn main_loop(window: Window, mut events_loop: EventsLoop, app: &mut App) {
    events_loop.run_forever(|event| {
        match event {
            Event::WindowEvent { event: WindowEvent::Closed, .. } => {
                println!("Vulkan window closing...");
                ControlFlow::Break
            },
            _ => ControlFlow::Continue,
        }
    });
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {

    }
}
