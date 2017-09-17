



use std::sync::Arc;
use std::ptr;
use std::ffi::CStr;
use vk;
use ::{util, Device, ShaderModule, PipelineLayout, RenderPass};



struct Inner {
    handle: vk::Pipeline,
    device: Device,
}

pub struct GraphicsPipeline {
    inner: Arc<Inner>,
}

impl GraphicsPipeline {
    pub fn new(device: Device, pipeline_layout: &PipelineLayout,
            render_pass: &RenderPass, swap_chain_extent: vk::Extent2D) -> GraphicsPipeline
    {
        let vert_shader_code = util::read_file("/src/vkc/shaders/vert.spv");
        let frag_shader_code = util::read_file("/src/vkc/shaders/frag.spv");

        let vert_shader_module = ShaderModule::new(device.clone(), &vert_shader_code);
        let frag_shader_module = ShaderModule::new(device.clone(), &frag_shader_code);

        let fn_name = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") };

        let vert_shader_stage_info = vk::PipelineShaderStageCreateInfo {
            sType: vk::STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            stage: vk::SHADER_STAGE_VERTEX_BIT,
            module: vert_shader_module.handle(),
            pName: fn_name.as_ptr(),
            pSpecializationInfo: ptr::null(),
        };

        let frag_shader_stage_info = vk::PipelineShaderStageCreateInfo {
            sType: vk::STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            stage: vk::SHADER_STAGE_FRAGMENT_BIT,
            module: frag_shader_module.handle(),
            pName: fn_name.as_ptr(),
            pSpecializationInfo: ptr::null(),
        };

        let shader_stages = [vert_shader_stage_info, frag_shader_stage_info];

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo {
            sType: vk::STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            vertexBindingDescriptionCount: 0,
            pVertexBindingDescriptions: ptr::null(),
            vertexAttributeDescriptionCount: 0,
            pVertexAttributeDescriptions: ptr::null(),
        };

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo {
            sType: vk::STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            topology: vk::PRIMITIVE_TOPOLOGY_TRIANGLE_LIST,
            primitiveRestartEnable: vk::FALSE,
        };

        let viewport = vk::Viewport {
            x: 0.0f32,
            y: 0.0f32,
            width: swap_chain_extent.width as f32,
            height: swap_chain_extent.height as f32,
            minDepth: 0.0f32,
            maxDepth: 1.0f32,
        };

        let scissor = vk::Rect2D {
            offset: vk::Offset2D {
                x: 0,
                y: 0,
            },
            extent: swap_chain_extent,
        };

        let viewport_state = vk::PipelineViewportStateCreateInfo {
            sType: vk::STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            viewportCount: 1,
            pViewports: &viewport,
            scissorCount: 1,
            pScissors: &scissor,
        };

        let rasterizer = vk::PipelineRasterizationStateCreateInfo {
            sType: vk::STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            depthClampEnable: vk::FALSE,
            rasterizerDiscardEnable: vk::FALSE,
            polygonMode: vk::POLYGON_MODE_FILL,
            cullMode: vk::CULL_MODE_BACK_BIT,
            frontFace: vk::FRONT_FACE_CLOCKWISE,
            depthBiasEnable: vk::FALSE,
            depthBiasConstantFactor: 0.0f32,
            depthBiasClamp: 0.0f32,
            depthBiasSlopeFactor: 0.0f32,
            lineWidth: 1.0f32,
        };

        let multisampling = vk::PipelineMultisampleStateCreateInfo {
            sType: vk::STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            rasterizationSamples: vk::SAMPLE_COUNT_1_BIT,
            sampleShadingEnable: vk::FALSE,
            minSampleShading: 1.0f32,
            pSampleMask: ptr::null(),
            alphaToCoverageEnable: vk::FALSE,
            alphaToOneEnable: vk::FALSE,
        };

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState {
            blendEnable: vk::FALSE,
            srcColorBlendFactor: vk::BLEND_FACTOR_ONE,
            dstColorBlendFactor: vk::BLEND_FACTOR_ZERO,
            colorBlendOp: vk::BLEND_OP_ADD,
            srcAlphaBlendFactor: vk::BLEND_FACTOR_ONE,
            dstAlphaBlendFactor: vk::BLEND_FACTOR_ZERO,
            alphaBlendOp: vk::BLEND_OP_ADD,
            colorWriteMask: vk::COLOR_COMPONENT_R_BIT | vk::COLOR_COMPONENT_G_BIT | vk::COLOR_COMPONENT_B_BIT | vk::COLOR_COMPONENT_A_BIT,
        };

        // ///////////////////////////////////
        // /////////// KEEPME (ALPHA BLENDING)
        // let color_blend_attachment = vk::PipelineColorBlendAttachmentState {
        //     blendEnable: vk::FALSE,
        //     srcColorBlendFactor: vk::BLEND_FACTOR_SRC_ALPHA,
        //     dstColorBlendFactor: vk::BLEND_FACTOR_ONE_MINUS_SRC_ALPHA,
        //     colorBlendOp: vk::BLEND_OP_ADD,
        //     srcAlphaBlendFactor: vk::BLEND_FACTOR_ONE,
        //     dstAlphaBlendFactor: vk::BLEND_FACTOR_ZERO,
        //     alphaBlendOp: vk::BLEND_OP_ADD,
        //     colorWriteMask: vk::COLOR_COMPONENT_R_BIT | vk::COLOR_COMPONENT_G_BIT | vk::COLOR_COMPONENT_B_BIT | vk::COLOR_COMPONENT_A_BIT,
        // }; ////////////////////////////////
        // ///////////////////////////////////

        let color_blending = vk::PipelineColorBlendStateCreateInfo {
            sType: vk::STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            logicOpEnable: vk::FALSE,
            logicOp: vk::LOGIC_OP_COPY,
            attachmentCount: 1,
            pAttachments: &color_blend_attachment,
            blendConstants: [0.0f32; 4],
        };

        let dynamic_states = [vk::DYNAMIC_STATE_VIEWPORT, vk::DYNAMIC_STATE_LINE_WIDTH];

        let dynamic_state = vk::PipelineDynamicStateCreateInfo {
            sType: vk::STRUCTURE_TYPE_PIPELINE_DYNAMIC_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            dynamicStateCount: 2,
            pDynamicStates: dynamic_states.as_ptr(),
        };


        let create_info = vk::GraphicsPipelineCreateInfo {
            sType: vk::STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO,
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
            pDepthStencilState: ptr::null(),
            pColorBlendState: &color_blending,
            pDynamicState: &dynamic_state,
            layout: pipeline_layout.handle(),
            renderPass: render_pass.handle(),
            subpass: 0,
            basePipelineHandle: 0,
            basePipelineIndex: -1,
        };

        let mut handle = 0;
        unsafe {
            ::check(device.vk().CreateGraphicsPipelines(device.handle(), 0, 1, &create_info,
                ptr::null(), &mut handle));
        }

        GraphicsPipeline {
            inner: Arc::new(Inner {
                handle,
                device,
            })
        }
    }

    pub fn handle(&self) -> vk::ShaderModule {
        self.inner.handle
    }

    pub fn device(&self) -> &Device {
        &self.inner.device
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().DestroyPipeline(self.device.handle(), self.handle, ptr::null());
        }
    }
}