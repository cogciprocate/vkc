#![allow(dead_code)]

extern crate vkc;

use std::ptr;
use vkc::winit::{EventsLoop, WindowBuilder, Window, /*ControlFlow,*/ Event, WindowEvent};
use vkc::{vk, device, VkcResult, Version, Instance, Device, Surface, Swapchain, ImageView,
    PipelineLayout, RenderPass, GraphicsPipeline, Framebuffer, CommandPool, Semaphore};

fn main() {
    unsafe {
        let mut app = App::new().unwrap();
        app.main_loop().unwrap();
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

unsafe fn init_instance() -> VkcResult<Instance> {
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
    queue_family_flags: vk::Flags,
    device: Device,
    surface: Surface,
    swapchain: Option<Swapchain>,
    image_views: Option<Vec<ImageView>>,
    render_pass: Option<RenderPass>,
    pipeline_layout: PipelineLayout,
    graphics_pipeline: Option<GraphicsPipeline>,
    framebuffers: Option<Vec<Framebuffer>>,
    command_pool: CommandPool,
    command_buffers: Option<Vec<vk::CommandBuffer>>,
    image_available_semaphore: Semaphore,
    render_finished_semaphore: Semaphore,
}

impl App {
    pub unsafe fn new() -> VkcResult<App> {
        let instance = init_instance()?;
        let (window, events_loop) = init_window();
        let surface = Surface::new(instance.clone(), &window)?;
        let queue_family_flags = vk::QUEUE_GRAPHICS_BIT;
        let physical_device = device::choose_physical_device(&instance, &surface,
            queue_family_flags)?;
        let device = Device::new(instance.clone(), &surface, physical_device, queue_family_flags)?;
        let swapchain = Swapchain::new(surface.clone(), device.clone(), queue_family_flags,
            None, None)?;
        let image_views = vkc::create_image_views(&swapchain)?;
        let render_pass = RenderPass::new(device.clone(), swapchain.image_format())?;
        let pipeline_layout = PipelineLayout::new(device.clone())?;
        let graphics_pipeline = GraphicsPipeline::new(device.clone(), &pipeline_layout,
            &render_pass, swapchain.extent().clone())?;
        let framebuffers = vkc::create_framebuffers(&device, &render_pass,
            &image_views, swapchain.extent().clone())?;
        let command_pool = CommandPool::new(device.clone(), &surface, queue_family_flags)?;
        let command_buffers = vkc::create_command_buffers(&device, &command_pool, &render_pass,
            &graphics_pipeline, &framebuffers, swapchain.extent())?;
        let image_available_semaphore = Semaphore::new(device.clone())?;
        let render_finished_semaphore = Semaphore::new(device.clone())?;

        Ok(App {
            instance,
            window: window,
            events_loop: events_loop,
            queue_family_flags,
            device: device,
            surface: surface,
            swapchain: Some(swapchain),
            image_views: Some(image_views),
            render_pass: Some(render_pass),
            pipeline_layout,
            graphics_pipeline: Some(graphics_pipeline),
            framebuffers: Some(framebuffers),
            command_pool,
            command_buffers: Some(command_buffers),
            image_available_semaphore,
            render_finished_semaphore,
        })
    }

    fn cleanup_swapchain(&mut self) {
        self.swapchain = None;
        self.image_views = None;
        self.render_pass = None;
        self.graphics_pipeline = None;
        self.framebuffers = None;
        unsafe {
            self.device.vk().FreeCommandBuffers(self.device.handle(), self.command_pool.handle(),
                self.command_buffers.as_ref().unwrap().len() as u32,
                self.command_buffers.as_mut().unwrap().as_mut_ptr());
        }
        self.command_buffers = None;
    }

    fn recreate_swapchain(&mut self, current_extent: vk::Extent2D) -> VkcResult<()> {
        unsafe { vkc::check(self.device.vk().DeviceWaitIdle(self.device.handle())); }

        // TODO: Look into using the  `oldSwapChain` field in the
        // `SwapchainCreateInfoKHR` to recreate in-flight.
        self.swapchain = Some(Swapchain::new(self.surface.clone(), self.device.clone(),
            self.queue_family_flags, Some(current_extent), self.swapchain.take())?);
        self.image_views = Some(vkc::create_image_views(self.swapchain.as_ref().unwrap())?);
        self.render_pass = Some(RenderPass::new(self.device.clone(),
            self.swapchain.as_ref().unwrap().image_format())?);
        self.graphics_pipeline = Some(GraphicsPipeline::new(self.device.clone(),
            &self.pipeline_layout, self.render_pass.as_ref().unwrap(),
            self.swapchain.as_ref().unwrap().extent().clone())?);
        self.framebuffers = Some(vkc::create_framebuffers(&self.device,
            self.render_pass.as_ref().unwrap(), self.image_views.as_ref().unwrap(),
            self.swapchain.as_ref().unwrap().extent().clone())?);
        self.command_buffers = Some(vkc::create_command_buffers(&self.device, &self.command_pool,
            self.render_pass.as_ref().unwrap(), self.graphics_pipeline.as_ref().unwrap(),
            self.framebuffers.as_ref().unwrap(), self.swapchain.as_ref().unwrap().extent())?);
        Ok(())
    }

    fn main_loop(&mut self) -> VkcResult<()> {
        let mut exit = false;
        let mut recreate_swap = false;
        let mut current_extent = self.swapchain.as_ref().unwrap().extent().clone();

        loop {
            vkc::draw_frame(&self.device, self.swapchain.as_ref().unwrap(),
                &self.image_available_semaphore, &self.render_finished_semaphore,
                self.command_buffers.as_ref().unwrap())?;

            self.events_loop.poll_events(|event| {
                match event {
                    Event::WindowEvent { event: WindowEvent::Resized(w, h), .. } => {
                        current_extent = vk::Extent2D { width: w, height: h };
                        recreate_swap = true;
                        println!("The window was resized to {}x{}", w, h);
                    },
                    Event::WindowEvent { event: WindowEvent::Closed, .. } => {
                        println!("Vulkan window closing...");
                        exit = true;
                    },
                    _ => ()
                }
            });

            if recreate_swap {
                self.recreate_swapchain(current_extent.clone())?;
                recreate_swap = false;
            };
            if exit { break; }
        }

        unsafe { vkc::check(self.device.vk().DeviceWaitIdle(self.device.handle())); }
        Ok(())
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
