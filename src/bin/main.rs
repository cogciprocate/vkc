#![allow(dead_code)]

extern crate vkc;

use std::ptr;
use vkc::winit::{EventsLoop, WindowBuilder, Window, ControlFlow, Event, WindowEvent};
use vkc::{vk, device, VkcResult, Version, Instance, Device, Surface, Swapchain, ImageView,
    PipelineLayout, RenderPass, GraphicsPipeline, Framebuffer, CommandPool};

fn main() {
    unsafe {
        let mut app = App::new().unwrap();
        app.main_loop();
    }
    println!("Goodbye.");
}

fn init_window() -> (Window, EventsLoop) {
    let events_loop = EventsLoop::new();
    let window = WindowBuilder::new()
        .with_title("vkc - Hello Triangle")
        .build(&events_loop).unwrap();
    (window, events_loop)
}

unsafe fn init_instance() -> Instance {
    let app_name = b"Hello Triangle\0";
    let engine_name = b"No Engine\0";

    let app_info = vk::ApplicationInfo {
        sType: vk::STRUCTURE_TYPE_APPLICATION_INFO,
        pNext: ptr::null(),
        pApplicationName: app_name.as_ptr() as *const i8,
        applicationVersion: Version::new(1, 0, 0).into(),
        pEngineName: engine_name.as_ptr() as *const i8,
        engineVersion: Version::new(1, 0, 0).into(),
        apiVersion: Version::new(1, 0, 51).into(),
    };

    Instance::new(&app_info)
}

struct App {
    instance: Instance,
    window: Window,
    events_loop: EventsLoop,
    device: Device,
    surface: Surface,
    swapchain: Swapchain,
    image_views: Vec<ImageView>,
    render_pass: RenderPass,
    pipeline_layout: PipelineLayout,
    graphics_pipeline: GraphicsPipeline,
    framebuffers: Vec<Framebuffer>,
    command_pool: CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
}

impl App {
    pub unsafe fn new() -> VkcResult<App> {
        let instance = init_instance();
        let (window, events_loop) = init_window();
        let surface = Surface::new(instance.clone(), &window);
        let queue_family_flags = vk::QUEUE_GRAPHICS_BIT;
        let physical_device = device::choose_physical_device(&instance, &surface,
            queue_family_flags);
        let device = Device::new(instance.clone(), &surface, physical_device, queue_family_flags);
        let swapchain = Swapchain::new(surface.clone(), device.clone(), queue_family_flags);
        let image_views = vkc::create_image_views(&swapchain);
        let render_pass = RenderPass::new(device.clone(), swapchain.image_format());
        let pipeline_layout = PipelineLayout::new(device.clone());
        let graphics_pipeline = GraphicsPipeline::new(device.clone(), &pipeline_layout,
            &render_pass, swapchain.extent().clone());
        let framebuffers = vkc::create_framebuffers(&device, &render_pass,
            &image_views, swapchain.extent().clone());
        let command_pool = CommandPool::new(device.clone(), &surface, queue_family_flags)?;
        let command_buffers = vkc::create_command_buffers(&device, &command_pool, &render_pass,
            &graphics_pipeline, &framebuffers, swapchain.extent())?;

        Ok(App {
            instance,
            window: window,
            events_loop: events_loop,
            device: device,
            surface: surface,
            swapchain,
            image_views,
            render_pass,
            pipeline_layout,
            graphics_pipeline,
            framebuffers,
            command_pool,
            command_buffers,
        })
    }

    unsafe fn main_loop(&mut self) {
        self.events_loop.run_forever(|event| {
            match event {
                Event::WindowEvent { event: WindowEvent::Closed, .. } => {
                    println!("Vulkan window closing...");
                    ControlFlow::Break
                },
                _ => ControlFlow::Continue,
            }
        });

        // self.events_loop.take();
        // self.surface.take();
    }
}

impl Drop for App {
    fn drop(&mut self) {
        // unsafe { self.instance.vk.DestroyInstance(self.instance.instance, ptr::null()); }
        // self.surface.take();
        // self.events_loop.take();
        // self.window.take();
        // self.device.take();
        println!("Goodbye Triangle...");
    }
}
