#![allow(unused_imports, dead_code, unused_variables)]

extern crate vkc;

use std::mem;
use std::ptr;
// use vkc::vk;
use vkc::winit::{EventsLoop, WindowBuilder, Window, /*ControlFlow,*/ Event, WindowEvent};
use vkc::{vk, device, VkcResult, Version, Instance, Device, Surface, Swapchain, ImageView,
    PipelineLayout, RenderPass, GraphicsPipeline, Framebuffer, CommandPool, Semaphore,
    Buffer, DeviceMemory};

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

fn init_instance() -> VkcResult<Instance> {
    let app_name = b"Hello Triangle\0";
    let engine_name = b"No Engine\0";

    let app_info = vk::VkApplicationInfo {
        sType: vk::VK_STRUCTURE_TYPE_APPLICATION_INFO,
        pNext: ptr::null(),
        pApplicationName: app_name.as_ptr() as *const i8,
        applicationVersion: Version::new(1, 0, 0).into(),
        pEngineName: engine_name.as_ptr() as *const i8,
        engineVersion: Version::new(1, 0, 0).into(),
        apiVersion: Version::new(1, 0, 51).into(),
    };

    unsafe { Instance::new(&app_info) }
}



fn find_memory_type(device: &Device, type_filter: u32, properties: vk::VkMemoryPropertyFlags) -> u32 {
    let mut mem_properties: vk::VkPhysicalDeviceMemoryProperties;
    unsafe {
        mem_properties = mem::uninitialized();
        device.instance().vk().core.vkGetPhysicalDeviceMemoryProperties(device.physical_device(),
            &mut mem_properties);
    }

    for i in 0..mem_properties.memoryTypeCount {
        if (type_filter & (1 << i)) != 0 &&
            (mem_properties.memoryTypes[i as usize].propertyFlags & properties) == properties
        {
            return i;
        }
    }
    panic!("Failed to find suitable memory type.");
}


fn create_vertex_buffer(device: &Device) -> VkcResult<(Buffer, DeviceMemory)> {
    let vertex_buffer = Buffer::new(device.clone())?;

    let mut mem_requirements: vk::VkMemoryRequirements;
    unsafe {
        mem_requirements = mem::uninitialized();
        device.vk().core.vkGetBufferMemoryRequirements(device.handle(), vertex_buffer.handle(),
            &mut mem_requirements);
    }

    let alloc_info = vk::VkMemoryAllocateInfo {
        sType: vk::VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
        pNext: ptr::null(),
        allocationSize: mem_requirements.size,
        memoryTypeIndex: find_memory_type(device, mem_requirements.memoryTypeBits,
            vk::VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | vk::VK_MEMORY_PROPERTY_HOST_COHERENT_BIT),
    };

    let mem_type_idx = find_memory_type(device, mem_requirements.memoryTypeBits,
            vk::VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | vk::VK_MEMORY_PROPERTY_HOST_COHERENT_BIT);
    let vertex_buffer_memory = DeviceMemory::new(device.clone(), mem_requirements.size,
        mem_type_idx)?;

    unsafe {
        vkc::check(device.vk().core.vkBindBufferMemory(device.handle(), vertex_buffer.handle(),
            vertex_buffer_memory.handle(), 0));
    }

    // void* data;
    // vkMapMemory(device, vertexBufferMemory, 0, bufferInfo.size, 0, &data);
    //     memcpy(data, vertices.data(), (size_t) bufferInfo.size);
    // vkUnmapMemory(device, vertexBufferMemory);

    Ok((vertex_buffer, vertex_buffer_memory))
}

struct App {
    instance: Instance,
    window: Window,
    events_loop: EventsLoop,
    queue_family_flags: vk::VkQueueFlags,
    device: Device,
    surface: Surface,
    swapchain: Option<Swapchain>,
    image_views: Option<Vec<ImageView>>,
    render_pass: Option<RenderPass>,
    pipeline_layout: PipelineLayout,
    graphics_pipeline: Option<GraphicsPipeline>,
    framebuffers: Option<Vec<Framebuffer>>,
    command_pool: CommandPool,
    command_buffers: Option<Vec<vk::VkCommandBuffer>>,
    image_available_semaphore: Semaphore,
    render_finished_semaphore: Semaphore,
    vertex_buffer: Buffer,
    vertex_buffer_memory: DeviceMemory,
}

impl App {
    #[allow(unused_unsafe)]
    pub unsafe fn new() -> VkcResult<App> {
        let instance = init_instance()?;
        let (window, events_loop) = init_window();
        let surface = Surface::new(instance.clone(), &window)?;
        let queue_family_flags = vk::VK_QUEUE_GRAPHICS_BIT;
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
        let (vertex_buffer, vertex_buffer_memory) = create_vertex_buffer(&device)?;

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
            vertex_buffer,
            vertex_buffer_memory,
        })
    }

    fn cleanup_swapchain(&mut self) {
        self.swapchain = None;
        self.image_views = None;
        self.render_pass = None;
        self.graphics_pipeline = None;
        self.framebuffers = None;
        unsafe {
            self.device.vk().core.vkFreeCommandBuffers(self.device.handle(), self.command_pool.handle(),
                self.command_buffers.as_ref().unwrap().len() as u32,
                self.command_buffers.as_mut().unwrap().as_mut_ptr());
        }
        self.command_buffers = None;
    }

    fn recreate_swapchain(&mut self, current_extent: vk::VkExtent2D) -> VkcResult<()> {
        unsafe { vkc::check(self.device.vk().core.vkDeviceWaitIdle(self.device.handle())); }

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

    pub fn draw_frame(&mut self) -> VkcResult<()> {
        let mut image_index = 0u32;
        let acq_res = unsafe {
            self.device.vk().khr_swapchain.vkAcquireNextImageKHR(self.device.handle(), self.swapchain.as_ref().unwrap().handle(),
                u64::max_value(), self.image_available_semaphore.handle(), 0, &mut image_index)
        };

        if acq_res == vk::VK_ERROR_OUT_OF_DATE_KHR {
            let dims = self.window.get_inner_size_pixels().unwrap();
            self.recreate_swapchain(vk::VkExtent2D { height: dims.0, width: dims.1 } )?;
            return Ok(());
        } else if acq_res != vk::VK_SUCCESS && acq_res != vk::VK_SUBOPTIMAL_KHR {
            panic!("Unable to present swap chain image");
        }

        let wait_semaphores = [self.image_available_semaphore.handle()];
        let wait_stages = [vk::VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT];
        let signal_semaphores = [self.render_finished_semaphore.handle()];

        let submit_info = vk::VkSubmitInfo {
            sType: vk::VK_STRUCTURE_TYPE_SUBMIT_INFO,
            pNext: ptr::null(),
            waitSemaphoreCount: wait_semaphores.len() as u32,
            pWaitSemaphores: wait_semaphores.as_ptr(),
            pWaitDstStageMask: wait_stages.as_ptr(),
            commandBufferCount: 1,
            pCommandBuffers: self.command_buffers.as_ref().unwrap()
                .get(image_index as usize).unwrap(),
            signalSemaphoreCount: signal_semaphores.len() as u32,
            pSignalSemaphores: signal_semaphores.as_ptr(),
        };

        unsafe { vkc::check(self.device.vk().core.vkQueueSubmit(self.device.queue(0), 1,
            &submit_info, 0)); }

        let swapchains = [self.swapchain.as_ref().unwrap().handle()];

        let present_info = vk::VkPresentInfoKHR {
            sType: vk::VK_STRUCTURE_TYPE_PRESENT_INFO_KHR,
            pNext: ptr::null(),
            waitSemaphoreCount: signal_semaphores.len() as u32,
            pWaitSemaphores: signal_semaphores.as_ptr(),
            swapchainCount: swapchains.len() as u32,
            pSwapchains: swapchains.as_ptr(),
            pImageIndices: &image_index,
            pResults: ptr::null_mut(),
        };

        unsafe {
            vkc::check(self.device.vk().khr_swapchain.vkQueuePresentKHR(self.device.queue(0), &present_info));
            vkc::check(self.device.vk().core.vkQueueWaitIdle(self.device.queue(0)));
        }

        Ok(())
    }

    fn main_loop(&mut self) -> VkcResult<()> {
        let mut exit = false;
        let mut recreate_swap = false;
        let mut current_extent = self.swapchain.as_ref().unwrap().extent().clone();

        loop {
            self.draw_frame()?;

            self.events_loop.poll_events(|event| {
                match event {
                    Event::WindowEvent { event: WindowEvent::Resized(w, h), .. } => {
                        current_extent = vk::VkExtent2D { width: w, height: h };
                        recreate_swap = true;
                        // println!("The window was resized to {}x{}", w, h);
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

        unsafe { vkc::check(self.device.vk().core.vkDeviceWaitIdle(self.device.handle())); }
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
