
/// A set of utility functions for the creation 
/// of pipelines involving compute shaders 
pub mod compute_pipeline {
    use std::sync::Arc;
    use vulkano::{descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet}, pipeline::{compute::ComputePipelineCreateInfo, layout::PipelineDescriptorSetLayoutCreateInfo, ComputePipeline, Pipeline, PipelineLayout, PipelineShaderStageCreateInfo}, shader::ShaderModule};

    use crate::vk_renderer::Vk;
    
    pub fn compute_pipeline(vk: Arc<Vk>, shader: Arc<ShaderModule>) -> Arc<ComputePipeline> {
        let compute_shader = shader.entry_point("main").unwrap();
        let stage = PipelineShaderStageCreateInfo::new(compute_shader);
        let layout = PipelineLayout::new(
            vk.device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages([&stage])
                .into_pipeline_layout_create_info(vk.device.clone())
                .unwrap(),
        )
        .unwrap();

        ComputePipeline::new(
            vk.device.clone(),
            None,
            ComputePipelineCreateInfo::stage_layout(stage, layout),
        )
        .expect("failed to create compute pipeline")
    }

    pub fn descriptor_set(
        vk: Arc<Vk>, 
        compute_pipeline: Arc<ComputePipeline>,
        writes: impl IntoIterator<Item = WriteDescriptorSet>,
    ) -> (Arc<PersistentDescriptorSet>, usize) {
        let descriptor_set_allocator = vk.allocators.descriptor_set.clone();
        let pipeline_layout = compute_pipeline.layout();
        let descriptor_set_layouts = pipeline_layout.set_layouts();

        let descriptor_set_layout_index = 0;
        let descriptor_set_layout = descriptor_set_layouts
            .get(descriptor_set_layout_index)
            .unwrap();
        (PersistentDescriptorSet::new(
            &descriptor_set_allocator,
            descriptor_set_layout.clone(),
            writes,
            [],
        )
        .unwrap(), descriptor_set_layout_index)
    }
}

/// A set of utility functions for the creation 
/// of pipelines involving both vertex and
/// fragment shaders 
pub mod graphics_pipeline {
    use std::collections::{HashMap, HashSet};
    use std::hash::RandomState;
    use std::sync::Arc;

    use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
    use vulkano::format::Format;
    use vulkano::image::view::ImageView;
    use vulkano::image::{Image, ImageCreateInfo, ImageUsage};
    use vulkano::memory::allocator::AllocationCreateInfo;
    use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
    use vulkano::pipeline::graphics::depth_stencil::DepthStencilState;
    use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
    use vulkano::pipeline::graphics::multisample::MultisampleState;
    use vulkano::pipeline::graphics::rasterization::{CullMode, RasterizationState};
    use vulkano::pipeline::graphics::vertex_input::{Vertex as VulcanoVertex, VertexDefinition};
    use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
    use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
    use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
    use vulkano::pipeline::{DynamicState, GraphicsPipeline, Pipeline, PipelineLayout, PipelineShaderStageCreateInfo};
    use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass};
    use vulkano::shader::ShaderModule;
    use vulkano::swapchain::Swapchain;

    use crate::vk_renderer::vertex::Vertex;
    use crate::vk_renderer::Vk;

    pub fn render_pass(vk: Arc<Vk>, swapchain: Option<Arc<Swapchain>>) -> Arc<RenderPass> {
        /* 
        for now, single pass renderpasses will do the job.
        although for the future, i might want to implement 
        deferred rendering.

        For that, it would be interesting to use:
            ordered_passes_renderpass!()
        */
        
        vulkano::single_pass_renderpass!(vk.device.clone(),
            attachments: {
                color_attachment: {
                    format: match swapchain {
                        Some(swapchain) => swapchain.image_format(),
                        None => Format::R8G8B8A8_UNORM,
                    },
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },

                depth_attachment: {
                    format: Format::D16_UNORM,
                    samples: 1,
                    load_op: Clear,
                    store_op: DontCare,
                }
            },
            pass: {
                color: [color_attachment],
                depth_stencil: {depth_attachment},
            },
        )
        .unwrap()
    }

    pub fn framebuffer(rp: Arc<RenderPass>, image: Arc<Image>) -> Arc<Framebuffer> {
        let view = ImageView::new_default(image.clone()).unwrap();

        Framebuffer::new(
            rp.clone(),
            FramebufferCreateInfo {
                attachments: vec![view],
                ..Default::default()
            },
        )
        .unwrap()
    }

    pub fn framebuffers(
        vk: Arc<Vk>,
        rp: Arc<RenderPass>, 
        images: &Vec<Arc<Image>>
    ) -> Vec<Arc<Framebuffer>> {
        let depth_image = ImageView::new_default(
                Image::new(
                vk.allocators.memory.clone(), 
                ImageCreateInfo {
                    format: Format::D16_UNORM,
                    extent: images[0].extent(),
                    usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
                    ..Default::default()
                }, 
                AllocationCreateInfo::default(),
            )
            .unwrap()
        )
        .unwrap();

        images
            .iter()
            .map(|image| {
                let view = ImageView::new_default(image.clone()).unwrap();
                Framebuffer::new(
                    rp.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![view, depth_image.clone()],
                        ..Default::default()
                    },
                )
                .unwrap()
            })
            .collect::<Vec<_>>()
    }

    #[allow(deprecated)]
    pub fn graphics_pipeline(
        vk: Arc<Vk>,
        vs: Arc<ShaderModule>,
        fs: Arc<ShaderModule>,
        rp: Arc<RenderPass>,
        vp: Viewport
    ) -> Arc<GraphicsPipeline> {
        let vs_entry = vs.entry_point("main").unwrap();
        let fs_entry = fs.entry_point("main").unwrap();

        let vertex_input_state = Vertex::per_vertex()
            .definition(&vs_entry.info().input_interface)
            .unwrap();

        let stages = [
            PipelineShaderStageCreateInfo::new(vs_entry),
            PipelineShaderStageCreateInfo::new(fs_entry),
        ];

        let layout = PipelineLayout::new(
            vk.device.clone(), 
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(vk.device.clone())
                .unwrap(),
        )
        .unwrap();

        let subpass = Subpass::from(rp.clone(), 0).unwrap();

        let pipeline = GraphicsPipeline::new(
            vk.device.clone(), 
            None, 
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),

                vertex_input_state: Some(vertex_input_state),

                input_assembly_state: Some(InputAssemblyState::default()),

                viewport_state: Some(ViewportState {
                    viewports: [vp].into_iter().collect(),
                    ..Default::default()
                }),

                rasterization_state: Some(RasterizationState {
                    cull_mode: CullMode::None,
                    ..Default::default()
                }),
                multisample_state: Some(MultisampleState::default()),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    subpass.num_color_attachments(),
                    ColorBlendAttachmentState::default(),
                )),

                depth_stencil_state: Some(DepthStencilState::simple_depth_test()),

                subpass: Some(subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(layout)
            },
        )
        .unwrap();

        let state = pipeline.dynamic_state();

        dbg!(state.get(&DynamicState::CullMode));

        pipeline
    }

    pub fn descriptor_set(
        vk: Arc<Vk>, 
        graphics_pipeline: Arc<GraphicsPipeline>,
        writes: impl IntoIterator<Item = WriteDescriptorSet>,
    ) -> (Arc<PersistentDescriptorSet>, usize) {
        let descriptor_set_allocator = vk.allocators.descriptor_set.clone();
        let pipeline_layout = graphics_pipeline.layout();
        let descriptor_set_layouts = pipeline_layout.set_layouts();

        let descriptor_set_layout_index = 0;
        let descriptor_set_layout = descriptor_set_layouts
            .get(descriptor_set_layout_index)
            .unwrap();
        (PersistentDescriptorSet::new(
            &descriptor_set_allocator,
            descriptor_set_layout.clone(),
            writes,
            [],
        )
        .unwrap(), descriptor_set_layout_index)
    }
}

pub mod vertex_shader {
    vulkano_shaders::shader!{
        ty: "vertex",
        src: r"
            #version 460

            layout(location = 0) in vec2 pos;

            layout(set = 0, binding = 0) uniform UniformBuffer {
                mat4 model;
                mat4 view;
                mat4 proj;
            } ubo;

            void main() {
                gl_Position = ubo.model * ubo.view * ubo.proj * vec4(pos, 0.0, 1.0);
            }
        ",
    }
}

pub mod fragment_shader {
    vulkano_shaders::shader!{
        ty: "fragment",
        src: r"
            #version 460

            layout(location = 0) out vec4 f_color;

            void main() {
                f_color = vec4(1.0, 0.0, 0.0, 1.0);
            }
        ",
    }
}

pub mod mandelbrot_shader {
    vulkano_shaders::shader!{
        ty: "compute",
        src: r"
            #version 460

            layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

            layout(set = 0, binding = 0, rgba8) uniform writeonly image2D img;

            void main() {
                vec2 uv = (gl_GlobalInvocationID.xy + vec2(0.5)) / vec2(imageSize(img));

                vec2 c = (uv - vec2(0.5)) * 2.0 - vec2(1.0, 0.0);

                vec2 z = vec2(0.0, 0.0);
                float i;
                float iterations = 64;
                for (i = 0; i < iterations; i += 1) {
                    z = vec2(
                        z.x * z.x - z.y * z.y + c.x,
                        z.y * z.x + z.x * z.y + c.y
                    );

                    if (length(z) > 4.0) {
                        break;
                    }
                }

                imageStore(img, ivec2(gl_GlobalInvocationID.xy), vec4(1-vec3(i/iterations), 1.0));
            }
        ",
    }
}
