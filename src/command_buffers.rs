use std::sync::Arc;
use std::ptr;
use vk;
use vks;
use ::{util, VkcResult, Device, Framebuffer, CommandPool, RenderPass, GraphicsPipeline};



// #[derive(Debug, Clone)]
// pub struct CommandBuffer {
//     handle: vk::VkCommandBuffer,
//     device: Device,
// }

// impl CommandBuffer {
//     pub fn new() -> VkcResult<CommandBuffer> {

//         let mut handle = 0;
//         unsafe {
//             ::check(device.vk().CreateCommandBuffer(device.handle(), &create_info,
//                 ptr::null(), &mut handle));
//         }

//         Ok(CommandBuffer {
//             handle,
//             device,
//         })
//     }

//     pub fn handle(&self) -> vk::VkCommandBuffer {
//         self.inner.handle
//     }

//     pub fn device(&self) -> &Device {
//         &self.inner.device
//     }
// }

    // void createCommandBuffers() {
    //     commandBuffers.resize(swapChainFramebuffers.size());

    //     VkCommandBufferAllocateInfo allocInfo = {};
    //     allocInfo.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO;
    //     allocInfo.commandPool = commandPool;
    //     allocInfo.level = VK_COMMAND_BUFFER_LEVEL_PRIMARY;
    //     allocInfo.commandBufferCount = (uint32_t) commandBuffers.size();

    //     if (vkAllocateCommandBuffers(device, &allocInfo, commandBuffers.data()) != VK_SUCCESS) {
    //         throw std::runtime_error("failed to allocate command buffers!");
    //     }

    //     for (size_t i = 0; i < commandBuffers.size(); i++) {
    //         VkCommandBufferBeginInfo beginInfo = {};
    //         beginInfo.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO;
    //         beginInfo.flags = VK_COMMAND_BUFFER_USAGE_SIMULTANEOUS_USE_BIT;

    //         vkBeginCommandBuffer(commandBuffers[i], &beginInfo);

    //         VkRenderPassBeginInfo renderPassInfo = {};
    //         renderPassInfo.sType = VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO;
    //         renderPassInfo.renderPass = renderPass;
    //         renderPassInfo.framebuffer = swapChainFramebuffers[i];
    //         renderPassInfo.renderArea.offset = {0, 0};
    //         renderPassInfo.renderArea.extent = swapChainExtent;

    //         VkClearValue clearColor = {0.0f, 0.0f, 0.0f, 1.0f};
    //         renderPassInfo.clearValueCount = 1;
    //         renderPassInfo.pClearValues = &clearColor;

    //         vkCmdBeginRenderPass(commandBuffers[i], &renderPassInfo, VK_SUBPASS_CONTENTS_INLINE);

    //             vkCmdBindPipeline(commandBuffers[i], VK_PIPELINE_BIND_POINT_GRAPHICS, graphicsPipeline);

    //             vkCmdDraw(commandBuffers[i], 3, 1, 0, 0);

    //         vkCmdEndRenderPass(commandBuffers[i]);

    //         if (vkEndCommandBuffer(commandBuffers[i]) != VK_SUCCESS) {
    //             throw std::runtime_error("failed to record command buffer!");
    //         }
    //     }
    // }


pub fn create_command_buffers(device: &Device, command_pool: &CommandPool,
        render_pass: &RenderPass, graphics_pipeline: &GraphicsPipeline,
        swapchain_framebuffers: &[Framebuffer], swapchain_extent: &vk::VkExtent2D)
        -> VkcResult<Vec<vk::VkCommandBuffer>>
{
    let mut command_buffers = Vec::with_capacity(swapchain_framebuffers.len());
    unsafe { command_buffers.set_len(swapchain_framebuffers.len()); }

    let alloc_info = vk::VkCommandBufferAllocateInfo {
        sType: vk::VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
        pNext: ptr::null(),
        commandPool: command_pool.handle(),
        // * COMMAND_BUFFER_LEVEL_PRIMARY: Can be submitted to a queue for
        //   execution, but cannot be called from other command buffers.
        // * COMMAND_BUFFER_LEVEL_SECONDARY: Cannot be submitted directly, but
        //   can be called from primary command buffers.
        level: vk::VK_COMMAND_BUFFER_LEVEL_PRIMARY,
        commandBufferCount: command_buffers.len() as u32,
    };

    unsafe {
        ::check(device.vk().core.vkAllocateCommandBuffers(device.handle(), &alloc_info,
            command_buffers.as_mut_ptr()));
    }

    for (&command_buffer, swapchain_framebuffer) in command_buffers.iter()
            .zip(swapchain_framebuffers.iter())
    {
        let begin_info = vk::VkCommandBufferBeginInfo {
            sType: vk::VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
            pNext: ptr::null(),
            // * COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT: The command buffer
            //   will be rerecorded right after executing it once.
            // * COMMAND_BUFFER_USAGE_RENDER_PASS_CONTINUE_BIT: This is a
            //   secondary command buffer that will be entirely within a
            //   single render pass.
            // * COMMAND_BUFFER_USAGE_SIMULTANEOUS_USE_BIT: The command buffer
            //   can be resubmitted while it is also already pending
            //   execution.
            flags: vk::VK_COMMAND_BUFFER_USAGE_SIMULTANEOUS_USE_BIT,
            pInheritanceInfo: ptr::null(),
        };

        unsafe {
            ::check(device.vk().core.vkBeginCommandBuffer(command_buffer, &begin_info));
        }

        let clear_color = vk::VkClearValue {
            color: vk::VkClearColorValue { float32: [0.0f32, 0.0f32, 0.0f32, 1.0f32] }
        };

        let render_pass_info = vk::VkRenderPassBeginInfo {
            sType: vk::VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO,
            pNext: ptr::null(),
            renderPass: render_pass.handle(),
            framebuffer:swapchain_framebuffer.handle(),
            renderArea: vk::VkRect2D {
                offset: vk::VkOffset2D { x: 0, y: 0, },
                extent: swapchain_extent.clone(),
            },
            clearValueCount: 1,
            pClearValues: &clear_color,
        };

        unsafe {
            device.vk().core.vkCmdBeginRenderPass(command_buffer, &render_pass_info,
                vk::VK_SUBPASS_CONTENTS_INLINE);
            device.vk().core.vkCmdBindPipeline(command_buffer, vk::VK_PIPELINE_BIND_POINT_GRAPHICS,
                graphics_pipeline.handle());
            // * vertexCount: Even though we don't have a vertex buffer, we
            //   technically still have 3 vertices to draw.
            // * instanceCount: Used for instanced rendering, use 1 if you're
            //   not doing that.
            // * firstVertex: Used as an offset into the vertex buffer,
            //   defines the lowest value of gl_VertexIndex.
            // * firstInstance: Used as an offset for instanced rendering,
            //   defines the lowest value of gl_InstanceIndex.
            device.vk().core.vkCmdDraw(command_buffer, 3, 1, 0, 0);
            device.vk().core.vkCmdEndRenderPass(command_buffer);
            device.vk().core.vkEndCommandBuffer(command_buffer);
        }
    }
    Ok(command_buffers)
}






// #[derive(Debug)]
// struct Inner {
//     handle: vk::VkCommandBuffer,
//     device: Device,
// }

// #[derive(Debug, Clone)]
// pub struct CommandBuffer {
//     inner: Arc<Inner>,
// }

// impl CommandBuffer {
//     pub fn new() -> VkcResult<CommandBuffer> {

//         let mut handle = 0;
//         unsafe {
//             ::check(device.vk().CreateCommandBuffer(device.handle(), &create_info,
//                 ptr::null(), &mut handle));
//         }

//         Ok(CommandBuffer {
//             inner: Arc::new(Inner {
//                 handle,
//                 device,
//             })
//         })
//     }

//     pub fn handle(&self) -> vk::VkCommandBuffer {
//         self.inner.handle
//     }

//     pub fn device(&self) -> &Device {
//         &self.inner.device
//     }
// }

// impl Drop for Inner {
//     fn drop(&mut self) {
//         unsafe {
//             self.device.vk().DestroyCommandBuffer(self.device.handle(), self.handle, ptr::null());
//         }
//     }
// }