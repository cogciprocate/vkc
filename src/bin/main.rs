#![allow(unused_imports, dead_code, unused_variables)]

extern crate vkc;
extern crate cgmath;
extern crate image;

use std::mem;
use std::ptr;
use std::time;
use image::{ImageFormat, DynamicImage};
use cgmath::{SquareMatrix, One, Rotation, Rotation3, Basis3, Matrix3, Matrix4, Vector3};
use vkc::winit::{EventsLoop, WindowBuilder, Window, Event, WindowEvent};
use vkc::{vk, util, device, VkcResult, Version, Instance, Device, Surface, Swapchain, ImageView,
    PipelineLayout, RenderPass, GraphicsPipeline, Framebuffer, CommandPool, Semaphore,
    Buffer, DeviceMemory, Vertex, DescriptorSetLayout, UniformBufferObject, DescriptorPool,
    Image, Sampler};


const VERTICES: [Vertex; 8] =  [
    Vertex { pos: [-0.5, -0.5, 0.0], color: [1.0, 0.0, 0.0], tex_coord: [1.0, 0.0]},
    Vertex { pos: [0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0], tex_coord: [0.0, 0.0] },
    Vertex { pos: [0.5, 0.5, 0.0], color: [0.0, 0.0, 1.0], tex_coord: [0.0, 1.0] },
    Vertex { pos: [-0.5, 0.5, 0.0], color: [1.0, 1.0, 1.0], tex_coord: [1.0, 1.0] },
    Vertex { pos: [-0.5, -0.5, -0.5], color: [1.0, 0.0, 0.0], tex_coord: [1.0, 0.0]},
    Vertex { pos: [0.5, -0.5, -0.5], color: [0.0, 1.0, 0.0], tex_coord: [0.0, 0.0] },
    Vertex { pos: [0.5, 0.5, -0.5], color: [0.0, 0.0, 1.0], tex_coord: [0.0, 1.0] },
    Vertex { pos: [-0.5, 0.5, -0.5], color: [1.0, 1.0, 1.0], tex_coord: [1.0, 1.0] },
];

const INDICES: [u16; 12] = [
    0, 1, 2, 2, 3, 0,
    4, 5, 6, 6, 7, 4
];


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

fn begin_single_time_commands(device: &Device, command_pool: &CommandPool)
        -> VkcResult<vk::VkCommandBuffer>
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

    let begin_info = vk::VkCommandBufferBeginInfo {
        sType: vk::VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
        pNext: ptr::null(),
        flags: vk::VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT,
        pInheritanceInfo: ptr::null(),
    };

    unsafe { device.vk().core.vkBeginCommandBuffer(command_buffer, &begin_info); }
    Ok(command_buffer)
}

fn end_single_time_commands(device: &Device, command_pool: &CommandPool,
        command_buffer: vk::VkCommandBuffer) -> VkcResult<()>
{
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

    Ok(())
}

fn has_stencil_component(format: vk::VkFormat) -> bool {
    format == vk::VK_FORMAT_D32_SFLOAT_S8_UINT || format == vk::VK_FORMAT_D24_UNORM_S8_UINT
}

fn transition_image_layout(device: &Device, command_pool: &CommandPool, image: &Image,
        format: vk::VkFormat, old_layout: vk::VkImageLayout, new_layout: vk::VkImageLayout)
         -> VkcResult<()>
{
    let command_buffer = begin_single_time_commands(device, command_pool)?;

    let subresource_range = vk::VkImageSubresourceRange {
        aspectMask: vk::VK_IMAGE_ASPECT_COLOR_BIT,
        baseMipLevel: 0,
        levelCount: 1,
        baseArrayLayer: 0,
        layerCount: 1,
    };

    let mut barrier = vk::VkImageMemoryBarrier {
        sType: vk::VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER,
        pNext: ptr::null(),
        srcAccessMask: 0,
        dstAccessMask: 0,
        oldLayout: old_layout,
        newLayout: new_layout,
        srcQueueFamilyIndex: vk::VK_QUEUE_FAMILY_IGNORED,
        dstQueueFamilyIndex: vk::VK_QUEUE_FAMILY_IGNORED,
        image: image.handle(),
        subresourceRange: subresource_range,
    };

    if new_layout == vk::VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL {
        barrier.subresourceRange.aspectMask = vk::VK_IMAGE_ASPECT_DEPTH_BIT;
        if has_stencil_component(format) {
            barrier.subresourceRange.aspectMask |= vk::VK_IMAGE_ASPECT_STENCIL_BIT;
        }
    } else {
        barrier.subresourceRange.aspectMask = vk::VK_IMAGE_ASPECT_COLOR_BIT;
    }

    let source_stage: vk::VkPipelineStageFlags;
    let destination_stage: vk::VkPipelineStageFlags;

    if old_layout == vk::VK_IMAGE_LAYOUT_UNDEFINED &&
            new_layout == vk::VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL
    {
        barrier.srcAccessMask = 0;
        barrier.dstAccessMask = vk::VK_ACCESS_TRANSFER_WRITE_BIT;
        source_stage = vk::VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT;
        destination_stage = vk::VK_PIPELINE_STAGE_TRANSFER_BIT;
    } else if old_layout == vk::VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL &&
            new_layout == vk::VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL
    {
        barrier.srcAccessMask = vk::VK_ACCESS_TRANSFER_WRITE_BIT;
        barrier.dstAccessMask = vk::VK_ACCESS_SHADER_READ_BIT;
        source_stage = vk::VK_PIPELINE_STAGE_TRANSFER_BIT;
        destination_stage = vk::VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT;
    } else if old_layout == vk::VK_IMAGE_LAYOUT_UNDEFINED &&
            new_layout == vk::VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL {
        barrier.srcAccessMask = 0;
        barrier.dstAccessMask = vk::VK_ACCESS_DEPTH_STENCIL_ATTACHMENT_READ_BIT |
            vk::VK_ACCESS_DEPTH_STENCIL_ATTACHMENT_WRITE_BIT;
        source_stage = vk::VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT;
        destination_stage = vk::VK_PIPELINE_STAGE_EARLY_FRAGMENT_TESTS_BIT;
    } else {
        panic!("unsupported layout transition");
    }

    unsafe {
        device.vk().vkCmdPipelineBarrier(
            command_buffer,
            source_stage, destination_stage,
            0,
            0, ptr::null(),
            0, ptr::null(),
            1, &barrier
        );
    }

    end_single_time_commands(device, command_pool, command_buffer)
}

fn copy_buffer_to_image(device: &Device, command_pool: &CommandPool, buffer: &Buffer,
        image: &Image, width: u32, height: u32)  -> VkcResult<()>
{
    let command_buffer = begin_single_time_commands(device, command_pool)?;

    let image_subresource_layers = vk::VkImageSubresourceLayers {
        aspectMask: vk::VK_IMAGE_ASPECT_COLOR_BIT,
        mipLevel: 0,
        baseArrayLayer: 0,
        layerCount: 1,
    };

    let region = vk::VkBufferImageCopy {
        bufferOffset: 0,
        bufferRowLength: 0,
        bufferImageHeight: 0,
        imageSubresource: image_subresource_layers,
        imageOffset: vk::VkOffset3D { x: 0, y: 0, z: 0 },
        imageExtent: vk::VkExtent3D { width, height, depth: 1 },
    };

    unsafe {
        device.vk().vkCmdCopyBufferToImage(
            command_buffer,
            buffer.handle(),
            image.handle(),
            vk::VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
            1,
            &region
        );
    }

    end_single_time_commands(device, command_pool, command_buffer)
}

fn copy_buffer(device: &Device, command_pool: &CommandPool, src_buffer: &Buffer,
        dst_buffer: &Buffer, size: vk::VkDeviceSize)  -> VkcResult<()>
{
    // unsafe { device.vk().core.vkBeginCommandBuffer(command_buffer, &begin_info); }

    // TODO: Look into creating a separate command pool with the
    // `VK_COMMAND_POOL_CREATE_TRANSIENT_BIT` flag for short lived command
    // buffers like this.
    let command_buffer = begin_single_time_commands(device, command_pool)?;

    let copy_region = vk::VkBufferCopy {
        srcOffset: 0,
        dstOffset: 0,
        size: size,
    };

    unsafe { device.vk().core.vkCmdCopyBuffer(command_buffer, src_buffer.handle(),
        dst_buffer.handle(), 1, &copy_region); }

    end_single_time_commands(device, command_pool, command_buffer)
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

    copy_buffer(device, command_pool, &staging_buffer, &vertex_buffer, buffer_bytes)?;

    Ok(vertex_buffer)
}

fn create_index_buffer(device: &Device, command_pool: &CommandPool) -> VkcResult<Buffer> {
    let buffer_bytes = (mem::size_of::<u16>() * INDICES.len()) as u64;

    let staging_buffer = Buffer::new(device.clone(), buffer_bytes,
        vk::VK_BUFFER_USAGE_TRANSFER_SRC_BIT, vk::VK_SHARING_MODE_EXCLUSIVE,
        vk::VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT)?;

    unsafe {
        let mut data = ptr::null_mut();
        vkc::check(device.vk().core.vkMapMemory(device.handle(),
            staging_buffer.device_memory().handle(), 0, buffer_bytes, 0, &mut data));
        ptr::copy_nonoverlapping(&INDICES, data as *mut _, buffer_bytes as usize);
        device.vk().core.vkUnmapMemory(device.handle(), staging_buffer.device_memory().handle());
    }

    let index_buffer = Buffer::new(device.clone(), buffer_bytes,
        vk::VK_BUFFER_USAGE_TRANSFER_DST_BIT | vk::VK_BUFFER_USAGE_INDEX_BUFFER_BIT,
        vk::VK_SHARING_MODE_EXCLUSIVE, vk::VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT)?;

    copy_buffer(device, command_pool, &staging_buffer, &index_buffer, buffer_bytes)?;

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

fn find_supported_format(device: &Device, candidates: &[vk::VkFormat], tiling: vk::VkImageTiling,
        features: vk::VkFormatFeatureFlags) -> VkcResult<vk::VkFormat>
{
    for &format in candidates {
        let mut props: vk::VkFormatProperties;
        unsafe {
            props = mem::uninitialized();
            device.instance().vk().vkGetPhysicalDeviceFormatProperties(device.physical_device(),
            format, &mut props);
        }

        if tiling == vk::VK_IMAGE_TILING_LINEAR &&
                (props.linearTilingFeatures & features) == features
        {
            return Ok(format);
        } else if tiling == vk::VK_IMAGE_TILING_OPTIMAL &&
                (props.optimalTilingFeatures & features) == features
        {
            return Ok(format);
        }
    }

    panic!("Failed to find supported format.")
}


fn find_depth_format(device: &Device) -> VkcResult<vk::VkFormat> {
    find_supported_format(device, &[vk::VK_FORMAT_D32_SFLOAT, vk::VK_FORMAT_D32_SFLOAT_S8_UINT,
        vk::VK_FORMAT_D24_UNORM_S8_UINT], vk::VK_IMAGE_TILING_OPTIMAL,
        vk::VK_FORMAT_FEATURE_DEPTH_STENCIL_ATTACHMENT_BIT)
}

fn create_depth_resources(device: &Device, command_pool: &CommandPool,
        swapchain_extent: vk::VkExtent2D) -> VkcResult<(Image, ImageView)>
{
    let depth_format = find_depth_format(device)?;
    let extent = vk::VkExtent3D { width: swapchain_extent.width,
        height: swapchain_extent.height, depth: 1 };

    let depth_image = Image::new(device.clone(), extent, depth_format, vk::VK_IMAGE_TILING_OPTIMAL,
        vk::VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT, vk::VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT)?;

    let depth_image_view = ImageView::new(device.clone(), None, depth_image.handle(), depth_format,
        vk::VK_IMAGE_ASPECT_DEPTH_BIT)?;

    transition_image_layout(device, command_pool, &depth_image, depth_format,
        vk::VK_IMAGE_LAYOUT_UNDEFINED, vk::VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL)?;

    Ok((depth_image, depth_image_view))
}

fn create_texture_image(device: &Device, command_pool: &CommandPool) -> VkcResult<Image> {
    let pixels = image::open("/src/vkc/textures/texture.jpg").unwrap().to_rgba();
    let (tex_width, tex_height) = pixels.dimensions();
    let image_bytes = (tex_width * tex_height * 4) as u64;

    let staging_buffer = Buffer::new(device.clone(), image_bytes,
        vk::VK_BUFFER_USAGE_TRANSFER_SRC_BIT, vk::VK_SHARING_MODE_EXCLUSIVE,
        vk::VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | vk::VK_MEMORY_PROPERTY_HOST_COHERENT_BIT)?;

    unsafe {
        let mut data = ptr::null_mut();
        vkc::check(device.vk().vkMapMemory(device.handle(),
            staging_buffer.device_memory().handle(), 0, image_bytes, 0, &mut data));
        ptr::copy_nonoverlapping(pixels.as_ptr(), data as *mut _, pixels.len());
        device.vk().vkUnmapMemory(device.handle(), staging_buffer.device_memory().handle());
    }

    let extent = vk::VkExtent3D { width: tex_width, height: tex_height, depth: 1 };
    let texture_image = Image::new(device.clone(), extent, vk::VK_FORMAT_R8G8B8A8_UNORM,
        vk::VK_IMAGE_TILING_OPTIMAL, vk::VK_IMAGE_USAGE_TRANSFER_DST_BIT |
        vk::VK_IMAGE_USAGE_SAMPLED_BIT, vk::VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT)?;

    transition_image_layout(device, command_pool, &texture_image, vk::VK_FORMAT_R8G8B8A8_UNORM,
        vk::VK_IMAGE_LAYOUT_UNDEFINED, vk::VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL)?;

    copy_buffer_to_image(device, command_pool, &staging_buffer, &texture_image,
        extent.width, extent.height)?;

    transition_image_layout(device, command_pool, &texture_image, vk::VK_FORMAT_R8G8B8A8_UNORM,
        vk::VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL, vk::VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL)?;

    Ok(texture_image)
}

fn create_texture_image_view(device: Device, image: &Image) -> VkcResult<ImageView> {
    ImageView::new(device, None, image.handle(), vk::VK_FORMAT_R8G8B8A8_UNORM,
        vk::VK_IMAGE_ASPECT_COLOR_BIT)
}

fn create_texture_sampler(device: Device) -> VkcResult<Sampler> {
    Sampler::new(device)
}

fn create_render_pass(device: Device, swapchain_image_format: vk::VkFormat)
        -> VkcResult<RenderPass>
{
    let depth_image_format = find_depth_format(&device)?;
    RenderPass::new(device.clone(), swapchain_image_format, depth_image_format)
}

fn create_descriptor_pool(device: Device) -> VkcResult<DescriptorPool> {
    DescriptorPool::new(device)
}

fn create_descriptor_set_layout(device: Device) -> VkcResult<DescriptorSetLayout> {
    DescriptorSetLayout::new(device)
}

fn create_descriptor_set(device: &Device, layout: &DescriptorSetLayout,
        pool: &DescriptorPool, uniform_buffer: &Buffer, texture_image_view: &ImageView,
        texture_sampler: &Sampler) -> VkcResult<vk::VkDescriptorSet>
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

    let image_info = vk::VkDescriptorImageInfo {
        sampler: texture_sampler.handle(),
        imageView: texture_image_view.handle(),
        imageLayout: vk::VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL,
    };

    let descriptor_writes = [
        vk::VkWriteDescriptorSet {
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
        },
        vk::VkWriteDescriptorSet {
            sType: vk::VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
            pNext: ptr::null(),
            dstSet: descriptor_set,
            dstBinding: 1,
            dstArrayElement: 0,
            descriptorCount: 1,
            descriptorType: vk::VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER,
            pImageInfo: &image_info,
            pBufferInfo: ptr::null(),
            pTexelBufferView: ptr::null(),
        },
    ];

    unsafe {
        device.vk().vkUpdateDescriptorSets(device.handle(), descriptor_writes.len() as u32,
            descriptor_writes.as_ptr(), 0, ptr::null());
    }

    Ok(descriptor_set)
}

struct SwapchainComponents {
    image_views: Vec<ImageView>,
    render_pass: RenderPass,
    graphics_pipeline: GraphicsPipeline,
    depth_image: Image,
    depth_image_view: ImageView,
    framebuffers: Vec<Framebuffer>,
}

struct App {
    instance: Instance,
    window: Window,
    events_loop: EventsLoop,
    queue_family_flags: vk::VkQueueFlags,
    device: Device,
    surface: Surface,
    descriptor_set_layout: DescriptorSetLayout,
    pipeline_layout: PipelineLayout,
    command_pool: CommandPool,
    texture_image: Image,
    texture_image_view: ImageView,
    texture_sampler: Sampler,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    uniform_buffer: Buffer,
    descriptor_pool: DescriptorPool,
    descriptor_set: vk::VkDescriptorSet,
    image_available_semaphore: Semaphore,
    render_finished_semaphore: Semaphore,
    start_time: time::Instant,
    swapchain: Option<Swapchain>,
    swapchain_components: Option<SwapchainComponents>,
    command_buffers: Option<Vec<vk::VkCommandBuffer>>,
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
        let render_pass = create_render_pass(device.clone(), swapchain.image_format())?;
        let descriptor_set_layout = create_descriptor_set_layout(device.clone())?;
        let pipeline_layout = PipelineLayout::new(device.clone(), Some(&descriptor_set_layout))?;
        let graphics_pipeline = GraphicsPipeline::new(device.clone(), &pipeline_layout,
            &render_pass, swapchain.extent().clone())?;
        let command_pool = CommandPool::new(device.clone(), &surface, queue_family_flags)?;
        let (depth_image, depth_image_view) = create_depth_resources(&device, &command_pool,
            swapchain.extent().clone())?;
        let framebuffers = vkc::create_framebuffers(&device, &render_pass,
            &image_views, &depth_image_view, swapchain.extent().clone())?;
        let texture_image = create_texture_image(&device, &command_pool)?;
        let texture_image_view = create_texture_image_view(device.clone(),
            &texture_image)?;
        let texture_sampler = create_texture_sampler(device.clone())?;
        let vertex_buffer = create_vertex_buffer(&device, &command_pool)?;
        let index_buffer = create_index_buffer(&device, &command_pool)?;
        let uniform_buffer = create_uniform_buffer(&device, &command_pool,
            swapchain.extent().clone())?;
        let descriptor_pool = create_descriptor_pool(device.clone())?;
        let descriptor_set = create_descriptor_set(&device, &descriptor_set_layout,
            &descriptor_pool, &uniform_buffer, &texture_image_view, &texture_sampler)?;
        let command_buffers = vkc::create_command_buffers(&device, &command_pool, &render_pass,
            &graphics_pipeline, &framebuffers, swapchain.extent(),
            &vertex_buffer, &index_buffer,
            VERTICES.len() as u32, INDICES.len() as u32, &pipeline_layout, descriptor_set)?;
        let image_available_semaphore = Semaphore::new(device.clone())?;
        let render_finished_semaphore = Semaphore::new(device.clone())?;
        let start_time = time::Instant::now();

        let swapchain_components = SwapchainComponents {
            image_views: image_views,
            render_pass: render_pass,
            graphics_pipeline: graphics_pipeline,
            depth_image,
            depth_image_view,
            framebuffers: framebuffers,
        };

        Ok(App {
            instance,
            window: window,
            events_loop: events_loop,
            queue_family_flags,
            device: device,
            surface: surface,
            descriptor_set_layout,
            pipeline_layout,
            command_pool,
            texture_image,
            texture_image_view,
            texture_sampler,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            descriptor_pool,
            descriptor_set,
            image_available_semaphore,
            render_finished_semaphore,
            start_time,
            swapchain: Some(swapchain),
            swapchain_components: Some(swapchain_components),
            command_buffers: Some(command_buffers),
        })
    }

    fn cleanup_swapchain(&mut self) {
        self.swapchain = None;
        self.swapchain_components = None;
        unsafe {
            self.device.vk().core.vkFreeCommandBuffers(self.device.handle(),
                self.command_pool.handle(),
                self.command_buffers.as_ref().unwrap().len() as u32,
                self.command_buffers.as_mut().unwrap().as_mut_ptr());
        }
        self.command_buffers = None;
    }

    fn recreate_swapchain(&mut self, current_extent: vk::VkExtent2D) -> VkcResult<()> {
        unsafe { vkc::check(self.device.vk().core.vkDeviceWaitIdle(self.device.handle())); }

        let swapchain = Swapchain::new(self.surface.clone(), self.device.clone(),
            self.queue_family_flags, Some(current_extent), self.swapchain.take())?;

        self.cleanup_swapchain();

        let image_views = vkc::create_image_views(&swapchain)?;
        let render_pass = create_render_pass(self.device.clone(),
            swapchain.image_format())?;
        let graphics_pipeline = GraphicsPipeline::new(self.device.clone(),
            &self.pipeline_layout, &render_pass,
            swapchain.extent().clone())?;
        let (depth_image, depth_image_view) = create_depth_resources(&self.device,
            &self.command_pool, swapchain.extent().clone())?;
        let framebuffers = vkc::create_framebuffers(&self.device,
            &render_pass, &image_views,
            &depth_image_view, swapchain.extent().clone())?;
        let command_buffers = vkc::create_command_buffers(&self.device, &self.command_pool,
            &render_pass, &graphics_pipeline,
            &framebuffers, swapchain.extent(),
            &self.vertex_buffer, &self.index_buffer, VERTICES.len() as u32,
            INDICES.len() as u32, &self.pipeline_layout, self.descriptor_set)?;

        self.swapchain = Some(swapchain);
        self.swapchain_components = Some(SwapchainComponents {
            image_views: image_views,
            render_pass: render_pass,
            graphics_pipeline: graphics_pipeline,
            depth_image,
            depth_image_view,
            framebuffers: framebuffers,
        });
        self.command_buffers = Some(command_buffers);

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
        println!("Goodbye Triangle...");
    }
}

