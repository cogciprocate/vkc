
use std::sync::Arc;
use std::ptr;
use std::ffi::CStr;
use vk;
use vks;
use ::{util, VkcResult, Device, ShaderModule, PipelineLayout, RenderPass, Vertex};



#[derive(Debug)]
struct Inner {
    handle: vk::VkPipeline,
    device: Device,
}

#[derive(Debug, Clone)]
pub struct GraphicsPipeline {
    inner: Arc<Inner>,
}

impl GraphicsPipeline {
    pub fn new(device: Device, pipeline_layout: &PipelineLayout,
            render_pass: &RenderPass, swap_chain_extent: vk::VkExtent2D, vert_shader_code: &[u8],
            frag_shader_code: &[u8]) -> VkcResult<GraphicsPipeline>
    {
        let vert_shader_module = ShaderModule::new(device.clone(), vert_shader_code)?;
        let frag_shader_module = ShaderModule::new(device.clone(), frag_shader_code)?;

        let fn_name = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") };

        let vert_shader_stage_info = vk::VkPipelineShaderStageCreateInfo {
            sType: vk::VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            stage: vk::VK_SHADER_STAGE_VERTEX_BIT,
            module: vert_shader_module.handle(),
            pName: fn_name.as_ptr(),
            pSpecializationInfo: ptr::null(),
        };

        let frag_shader_stage_info = vk::VkPipelineShaderStageCreateInfo {
            sType: vk::VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            stage: vk::VK_SHADER_STAGE_FRAGMENT_BIT,
            module: frag_shader_module.handle(),
            pName: fn_name.as_ptr(),
            pSpecializationInfo: ptr::null(),
        };

        let shader_stages = [vert_shader_stage_info, frag_shader_stage_info];

        let binding_description = Vertex::binding_description();
        let attribute_descriptions = Vertex::attribute_descriptions();

        let vertex_input_info = vk::VkPipelineVertexInputStateCreateInfo {
            sType: vk::VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            vertexBindingDescriptionCount: 1,
            pVertexBindingDescriptions: &binding_description,
            vertexAttributeDescriptionCount: attribute_descriptions.len() as u32,
            pVertexAttributeDescriptions: attribute_descriptions.as_ptr(),
        };

        let input_assembly = vk::VkPipelineInputAssemblyStateCreateInfo {
            sType: vk::VK_STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            // * VK_PRIMITIVE_TOPOLOGY_POINT_LIST: points from vertices
            // * VK_PRIMITIVE_TOPOLOGY_LINE_LIST: line from every 2 vertices
            //   without reuse
            // * VK_PRIMITIVE_TOPOLOGY_LINE_STRIP: the end vertex of every
            //   line is used as start vertex for the next line
            // * VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST: triangle from every 3
            //   vertices without reuse
            // * VK_PRIMITIVE_TOPOLOGY_TRIANGLE_STRIP: the second and third
            //   vertex of every triangle are used as first two vertices of
            //   the next triangle
            topology: vk::VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST,
            primitiveRestartEnable: vk::VK_FALSE,
        };

        let viewport = vk::VkViewport {
            x: 0.0f32,
            y: 0.0f32,
            width: swap_chain_extent.width as f32,
            height: swap_chain_extent.height as f32,
            minDepth: 0.0f32,
            maxDepth: 1.0f32,
        };

        let scissor = vk::VkRect2D {
            offset: vk::VkOffset2D {
                x: 0,
                y: 0,
            },
            extent: swap_chain_extent,
        };

        let viewport_state = vk::VkPipelineViewportStateCreateInfo {
            sType: vk::VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            viewportCount: 1,
            pViewports: &viewport,
            scissorCount: 1,
            pScissors: &scissor,
        };

        let rasterizer = vk::VkPipelineRasterizationStateCreateInfo {
            sType: vk::VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            depthClampEnable: vk::VK_FALSE,
            rasterizerDiscardEnable: vk::VK_FALSE,
            polygonMode: vk::VK_POLYGON_MODE_FILL,
            cullMode: vk::VK_CULL_MODE_BACK_BIT,
            // frontFace: vk::VK_FRONT_FACE_CLOCKWISE,
            frontFace: vk::VK_FRONT_FACE_COUNTER_CLOCKWISE,
            depthBiasEnable: vk::VK_FALSE,
            depthBiasConstantFactor: 0.0f32,
            depthBiasClamp: 0.0f32,
            depthBiasSlopeFactor: 0.0f32,
            lineWidth: 1.0f32,
        };

        let multisampling = vk::VkPipelineMultisampleStateCreateInfo {
            sType: vk::VK_STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            rasterizationSamples: vk::VK_SAMPLE_COUNT_1_BIT,
            sampleShadingEnable: vk::VK_FALSE,
            minSampleShading: 1.0f32,
            pSampleMask: ptr::null(),
            alphaToCoverageEnable: vk::VK_FALSE,
            alphaToOneEnable: vk::VK_FALSE,
        };

        let stencil_op_state = vk::VkStencilOpState {
            failOp: 0,
            passOp: 0,
            depthFailOp: 0,
            compareOp: 0,
            compareMask: 0,
            writeMask: 0,
            reference: 0,
        };

        let depth_stencil = vk::VkPipelineDepthStencilStateCreateInfo {
            sType: vk::VK_STRUCTURE_TYPE_PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            depthTestEnable: vk::VK_TRUE,
            depthWriteEnable: vk::VK_TRUE,
            depthCompareOp: vk::VK_COMPARE_OP_LESS,
            depthBoundsTestEnable: vk::VK_FALSE,
            stencilTestEnable: vk::VK_FALSE,
            front: stencil_op_state.clone(),
            back: stencil_op_state,
            minDepthBounds: 0.0,
            maxDepthBounds: 1.0,
        };

        let color_blend_attachment = vk::VkPipelineColorBlendAttachmentState {
            blendEnable: vk::VK_FALSE,
            srcColorBlendFactor: vk::VK_BLEND_FACTOR_ONE,
            dstColorBlendFactor: vk::VK_BLEND_FACTOR_ZERO,
            colorBlendOp: vk::VK_BLEND_OP_ADD,
            srcAlphaBlendFactor: vk::VK_BLEND_FACTOR_ONE,
            dstAlphaBlendFactor: vk::VK_BLEND_FACTOR_ZERO,
            alphaBlendOp: vk::VK_BLEND_OP_ADD,
            colorWriteMask: vk::VK_COLOR_COMPONENT_R_BIT | vk::VK_COLOR_COMPONENT_G_BIT |
                vk::VK_COLOR_COMPONENT_B_BIT | vk::VK_COLOR_COMPONENT_A_BIT,
        };

        // ///////////////////////////////////////////////
        // /////////// KEEPME (ALPHA BLENDING) ///////////
        // let color_blend_attachment = vk::VkPipelineColorBlendAttachmentState {
        //     blendEnable: vk::VK_FALSE,
        //     srcColorBlendFactor: vk::VK_BLEND_FACTOR_SRC_ALPHA,
        //     dstColorBlendFactor: vk::VK_BLEND_FACTOR_ONE_MINUS_SRC_ALPHA,
        //     colorBlendOp: vk::VK_BLEND_OP_ADD,
        //     srcAlphaBlendFactor: vk::VK_BLEND_FACTOR_ONE,
        //     dstAlphaBlendFactor: vk::VK_BLEND_FACTOR_ZERO,
        //     alphaBlendOp: vk::VK_BLEND_OP_ADD,
        //     colorWriteMask: vk::VK_COLOR_COMPONENT_R_BIT | vk::VK_COLOR_COMPONENT_G_BIT | vk::VK_COLOR_COMPONENT_B_BIT | vk::VK_COLOR_COMPONENT_A_BIT,
        // }; ////////////////////////////////////////////
        // ///////////////////////////////////////////////

        let color_blending = vk::VkPipelineColorBlendStateCreateInfo {
            sType: vk::VK_STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            logicOpEnable: vk::VK_FALSE,
            logicOp: vk::VK_LOGIC_OP_COPY,
            attachmentCount: 1,
            pAttachments: &color_blend_attachment,
            blendConstants: [0.0f32; 4],
        };

        // ///////////////////////////////////////////////
        // /////////// KEEPME (DYNAMIC STATES) ///////////
        // let dynamic_states = [vk::VK_DYNAMIC_STATE_VIEWPORT, vk::VK_DYNAMIC_STATE_LINE_WIDTH];
        // let dynamic_state = vk::VkPipelineDynamicStateCreateInfo {
        //     sType: vk::VK_STRUCTURE_TYPE_PIPELINE_DYNAMIC_STATE_CREATE_INFO,
        //     pNext: ptr::null(),
        //     flags: 0,
        //     dynamicStateCount: 2,
        //     pDynamicStates: dynamic_states.as_ptr(),
        // }; ////////////////////////////////////////////
        // ///////////////////////////////////////////////

        let create_info = vk::VkGraphicsPipelineCreateInfo {
            sType: vk::VK_STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            stageCount: 2,
            pStages: shader_stages.as_ptr(),
            pVertexInputState: &vertex_input_info,
            pInputAssemblyState: &input_assembly,
            pTessellationState: ptr::null(),
            pViewportState: &viewport_state,
            pRasterizationState: &rasterizer,
            pMultisampleState: &multisampling,
            pDepthStencilState: &depth_stencil,
            pColorBlendState: &color_blending,
            // pDynamicState: &dynamic_state,
            pDynamicState: ptr::null(),
            layout: pipeline_layout.handle(),
            renderPass: render_pass.handle(),
            subpass: 0,
            basePipelineHandle: 0,
            basePipelineIndex: -1,
        };

        let mut handle = 0;
        unsafe {
            ::check(device.vk().core.vkCreateGraphicsPipelines(device.handle(), 0, 1, &create_info,
                ptr::null(), &mut handle));
        }

        Ok(GraphicsPipeline {
            inner: Arc::new(Inner {
                handle,
                device,
            })
        })
    }

    pub fn handle(&self) -> vk::VkPipeline {
        self.inner.handle
    }

    pub fn device(&self) -> &Device {
        &self.inner.device
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().core.vkDestroyPipeline(self.device.handle(), self.handle, ptr::null());
        }
    }
}