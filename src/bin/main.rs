#![allow(unused_imports, dead_code, unused_variables)]

extern crate vkc;
extern crate cgmath;

use std::mem;
use std::ptr;
use std::time;
use cgmath::{SquareMatrix, One, Rotation, Rotation3, Basis3, Matrix3, Matrix4, Vector3};
use vkc::winit::{EventsLoop, WindowBuilder, Window, Event, WindowEvent};
use vkc::{vk, device, VkcResult, Version, Instance, Device, Surface, Swapchain, ImageView,
    PipelineLayout, RenderPass, GraphicsPipeline, Framebuffer, CommandPool, Semaphore,
    Buffer, DeviceMemory, Vertex, DescriptorSetLayout, UniformBufferObject, DescriptorPool};


const VERTICES: [Vertex; 4] =  [
    Vertex { pos: [-0.5f32, -0.5f32], color: [1.0f32, 0.0f32, 0.0f32] },
    Vertex { pos: [0.5f32, -0.5f32], color: [0.0f32, 1.0f32, 0.0f32] },
    Vertex { pos: [0.5f32, 0.5f32], color: [0.0f32, 0.0f32, 1.0f32] },
    Vertex { pos: [-0.5f32, 0.5f32], color: [1.0f32, 1.0f32, 1.0f32] },
];

const INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];


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

fn copy_buffer(device: &Device, command_pool: &CommandPool, src_buffer: &Buffer,
        dst_buffer: &Buffer, size: vk::VkDeviceSize)
{
    let alloc_info = vk::VkCommandBufferAllocateInfo {
        sType: vk::VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
        pNext: ptr::null(),
        commandPool: command_pool.handle(),
        level: vk::VK_COMMAND_BUFFER_LEVEL_PRIMARY,
        commandBufferCount: 1,
    };

    let mut command_buffer = ptr::null_mut();
    unsafe {
        vkc::check(device.vk().core.vkAllocateCommandBuffers(device.handle(), &alloc_info,
            &mut command_buffer));
    }

    // TODO: Look into creating a separate command pool with the
    // `VK_COMMAND_POOL_CREATE_TRANSIENT_BIT` flag for short lived command
    // buffers like this.

    let begin_info = vk::VkCommandBufferBeginInfo {
        sType: vk::VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
        pNext: ptr::null(),
        flags: vk::VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT,
        pInheritanceInfo: ptr::null(),
    };

    unsafe { device.vk().core.vkBeginCommandBuffer(command_buffer, &begin_info); }

    let copy_region = vk::VkBufferCopy {
        srcOffset: 0,
        dstOffset: 0,
        size: size,
    };

    unsafe { device.vk().core.vkCmdCopyBuffer(command_buffer, src_buffer.handle(),
        dst_buffer.handle(), 1, &copy_region); }

    unsafe { vkc::check(device.vk().core.vkEndCommandBuffer(command_buffer)); }

    let submit_info = vk::VkSubmitInfo {
        sType: vk::VK_STRUCTURE_TYPE_SUBMIT_INFO,
        pNext: ptr::null(),
        waitSemaphoreCount: 0,
        pWaitSemaphores: ptr::null(),
        pWaitDstStageMask: ptr::null(),
        commandBufferCount: 1,
        pCommandBuffers: &command_buffer,
        signalSemaphoreCount: 0,
        pSignalSemaphores: ptr::null(),
    };

    unsafe {
        vkc::check(device.vk().core.vkQueueSubmit(device.queue(0), 1,
            &submit_info, 0));
        vkc::check(device.vk().core.vkQueueWaitIdle(device.queue(0)));
        device.vk().core.vkFreeCommandBuffers(device.handle(),
            command_pool.handle(), 1, &command_buffer);
    }
}

fn create_vertex_buffer(device: &Device, command_pool: &CommandPool) -> VkcResult<Buffer> {
    // let buffer_bytes = (mem::size_of_val(&VERTICES[0]) * VERTICES.len()) as u64;
    let buffer_bytes = (mem::size_of::<[Vertex; 4]>() * VERTICES.len()) as u64;

    let staging_buffer = Buffer::new(device.clone(), buffer_bytes,
        vk::VK_BUFFER_USAGE_TRANSFER_SRC_BIT, vk::VK_SHARING_MODE_EXCLUSIVE,
        vk::VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | vk::VK_MEMORY_PROPERTY_HOST_COHERENT_BIT)?;

    let mut data = ptr::null_mut();
    unsafe {
        vkc::check(device.vk().core.vkMapMemory(device.handle(),
            staging_buffer.device_memory().handle(), 0, buffer_bytes, 0, &mut data));
        // println!("##### VBO data (ptr): {:?}", data);
        ptr::copy_nonoverlapping(&VERTICES, data as *mut _, buffer_bytes as usize);
        device.vk().core.vkUnmapMemory(device.handle(), staging_buffer.device_memory().handle());
    }

    // HOST-RW:
    // let vertex_buffer = Buffer::new(device.clone(), buffer_bytes,
    //     vk::VK_BUFFER_USAGE_VERTEX_BUFFER_BIT, vk::VK_SHARING_MODE_EXCLUSIVE,
    //     vk::VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | vk::VK_MEMORY_PROPERTY_HOST_COHERENT_BIT)?;
    let vertex_buffer = Buffer::new(device.clone(), buffer_bytes,
        vk::VK_BUFFER_USAGE_TRANSFER_DST_BIT | vk::VK_BUFFER_USAGE_VERTEX_BUFFER_BIT,
        vk::VK_SHARING_MODE_EXCLUSIVE, vk::VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT)?;

    copy_buffer(device, command_pool, &staging_buffer, &vertex_buffer, buffer_bytes);

    Ok(vertex_buffer)
}

fn create_index_buffer(device: &Device, command_pool: &CommandPool) -> VkcResult<Buffer> {
    let buffer_bytes = (mem::size_of::<u16>() * INDICES.len()) as u64;

    let staging_buffer = Buffer::new(device.clone(), buffer_bytes,
        vk::VK_BUFFER_USAGE_TRANSFER_SRC_BIT, vk::VK_SHARING_MODE_EXCLUSIVE,
        vk::VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT)?;

    let mut data = ptr::null_mut();
    unsafe {
        vkc::check(device.vk().core.vkMapMemory(device.handle(),
            staging_buffer.device_memory().handle(), 0, buffer_bytes, 0, &mut data));
        // println!("##### IBO data (ptr): {:?}", data);
        ptr::copy_nonoverlapping(&INDICES, data as *mut _, buffer_bytes as usize);
        device.vk().core.vkUnmapMemory(device.handle(), staging_buffer.device_memory().handle());
    }

    let index_buffer = Buffer::new(device.clone(), buffer_bytes,
        vk::VK_BUFFER_USAGE_TRANSFER_DST_BIT | vk::VK_BUFFER_USAGE_INDEX_BUFFER_BIT,
        vk::VK_SHARING_MODE_EXCLUSIVE, vk::VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT)?;

    copy_buffer(device, command_pool, &staging_buffer, &index_buffer, buffer_bytes);

    Ok(index_buffer)
}


fn create_uniform_buffer(device: &Device, command_pool: &CommandPool, _extent: vk::VkExtent2D)
        -> VkcResult<Buffer>
{
    let buffer_bytes = mem::size_of::<UniformBufferObject>() as u64;
    let uniform_buffer = Buffer::new(device.clone(), buffer_bytes,
        vk::VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT,
        vk::VK_SHARING_MODE_EXCLUSIVE, vk::VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT |
        vk::VK_MEMORY_PROPERTY_HOST_COHERENT_BIT)?;
    Ok(uniform_buffer)
}

fn create_descriptor_pool(device: Device) -> VkcResult<DescriptorPool>{
    DescriptorPool::new(device)
}

fn create_descriptor_set_layout(device: Device) -> VkcResult<DescriptorSetLayout> {
    DescriptorSetLayout::new(device)
}

fn create_descriptor_set(device: &Device, layout: &DescriptorSetLayout,
        pool: &DescriptorPool, uniform_buffer: &Buffer) -> VkcResult<vk::VkDescriptorSet>
{
    let layouts = [layout.handle()];

    let alloc_info = vk::VkDescriptorSetAllocateInfo {
        sType: vk::VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO,
        pNext: ptr::null(),
        descriptorPool: pool.handle(),
        descriptorSetCount: layouts.len() as u32,
        pSetLayouts: layouts.as_ptr(),
    };

    let mut descriptor_set = 0;
    unsafe {
        vkc::check(device.vk().vkAllocateDescriptorSets(device.handle(), &alloc_info,
            &mut descriptor_set));
    }

    let buffer_info = vk::VkDescriptorBufferInfo {
        buffer: uniform_buffer.handle(),
        offset: 0,
        range: mem::size_of::<UniformBufferObject>() as u64,
    };

    let descriptor_write = vk::VkWriteDescriptorSet {
        sType: vk::VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
        pNext: ptr::null(),
        dstSet: descriptor_set,
        dstBinding: 0,
        dstArrayElement: 0,
        descriptorCount: 1,
        descriptorType: vk::VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
        pImageInfo: ptr::null(),
        pBufferInfo: &buffer_info,
        pTexelBufferView: ptr::null(),
    };

    unsafe {
        device.vk().vkUpdateDescriptorSets(device.handle(), 1, &descriptor_write, 0, ptr::null());
    }

    Ok(descriptor_set)
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
    descriptor_set_layout: DescriptorSetLayout,
    pipeline_layout: PipelineLayout,
    graphics_pipeline: Option<GraphicsPipeline>,
    framebuffers: Option<Vec<Framebuffer>>,
    command_pool: CommandPool,
    command_buffers: Option<Vec<vk::VkCommandBuffer>>,
    image_available_semaphore: Semaphore,
    render_finished_semaphore: Semaphore,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    uniform_buffer: Buffer,
    descriptor_pool: DescriptorPool,
    descriptor_set: vk::VkDescriptorSet,
    start_time: time::Instant,
    // vertex_buffer_memory: DeviceMemory,
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
        let descriptor_set_layout = create_descriptor_set_layout(device.clone())?;
        let pipeline_layout = PipelineLayout::new(device.clone(), Some(&descriptor_set_layout))?;
        let graphics_pipeline = GraphicsPipeline::new(device.clone(), &pipeline_layout,
            &render_pass, swapchain.extent().clone())?;
        let framebuffers = vkc::create_framebuffers(&device, &render_pass,
            &image_views, swapchain.extent().clone())?;
        let command_pool = CommandPool::new(device.clone(), &surface, queue_family_flags)?;
        let vertex_buffer = create_vertex_buffer(&device, &command_pool)?;
        let index_buffer = create_index_buffer(&device, &command_pool)?;
        let uniform_buffer = create_uniform_buffer(&device, &command_pool,
            swapchain.extent().clone())?;
        let descriptor_pool = create_descriptor_pool(device.clone())?;
        let descriptor_set = create_descriptor_set(&device, &descriptor_set_layout,
            &descriptor_pool, &uniform_buffer)?;
        let command_buffers = vkc::create_command_buffers(&device, &command_pool, &render_pass,
            &graphics_pipeline, &framebuffers, swapchain.extent(),
            &vertex_buffer, &index_buffer,
            VERTICES.len() as u32, INDICES.len() as u32, &pipeline_layout, descriptor_set)?;
        let image_available_semaphore = Semaphore::new(device.clone())?;
        let render_finished_semaphore = Semaphore::new(device.clone())?;
        let start_time = time::Instant::now();

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
            descriptor_set_layout,
            pipeline_layout,
            graphics_pipeline: Some(graphics_pipeline),
            framebuffers: Some(framebuffers),
            command_pool,
            command_buffers: Some(command_buffers),
            image_available_semaphore,
            render_finished_semaphore,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            descriptor_pool,
            descriptor_set,
            start_time,
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
            self.framebuffers.as_ref().unwrap(), self.swapchain.as_ref().unwrap().extent(),
            &self.vertex_buffer, &self.index_buffer, VERTICES.len() as u32,
            INDICES.len() as u32, &self.pipeline_layout, self.descriptor_set)?);
        Ok(())
    }

    fn update_uniform_buffer(&mut self) -> VkcResult<()> {
        let current_time = time::Instant::now();
        let elapsed = current_time.duration_since(self.start_time);
        let time = elapsed.as_secs() as f32 + (elapsed.subsec_nanos() as f32 * 1e-9);

        let extent = self.swapchain.as_ref().unwrap().extent().clone();
        let mut proj = cgmath::perspective(cgmath::Rad(45.0f32.to_radians()),
            extent.width as f32 / extent.height as f32, 0.1, 10.0);
        let view = cgmath::Matrix4::look_at(cgmath::Point3::new(2.0, 2.0, 2.0),
            cgmath::Point3::new(0.0, 0.0, 0.0), cgmath::Vector3::new(0.0, 0.0, 1.0));
        let scale = cgmath::Matrix4::from_scale(1.0);
        proj[1][1] *= -1.0;

        let rotation = Matrix3::from_angle_z(cgmath::Rad(time));
        let model = Matrix4::from(rotation).into();

        let ubo = UniformBufferObject {
            model: model,
            view: (view * scale).into(),
            proj: proj.into(),
        };


        let mut data = ptr::null_mut();
        unsafe {
            vkc::check(self.device.vk().core.vkMapMemory(self.device.handle(),
                self.uniform_buffer.device_memory().handle(), 0,
                mem::size_of::<UniformBufferObject>() as u64, 0, &mut data));
            ptr::copy_nonoverlapping(&ubo, data as *mut _, 1);
            self.device.vk().core.vkUnmapMemory(self.device.handle(),
                self.uniform_buffer.device_memory().handle());
        }

        Ok(())
    }

    fn draw_frame(&mut self) -> VkcResult<()> {
        let mut image_index = 0u32;
        let acq_res = unsafe {
            self.device.vk().khr_swapchain.vkAcquireNextImageKHR(self.device.handle(),
                self.swapchain.as_ref().unwrap().handle(),
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

            self.update_uniform_buffer()?;
            self.draw_frame()?;
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

