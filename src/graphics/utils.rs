#![allow(unused, deprecated)]

use std::sync::Arc;

use vulkano::{descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet}, format::{ClearValue, Format}, image::{view::ImageView, Image, ImageCreateInfo, ImageUsage}, memory::allocator::AllocationCreateInfo, pipeline::{graphics::{color_blend::{ColorBlendAttachmentState, ColorBlendState}, depth_stencil::DepthStencilState, input_assembly::InputAssemblyState, multisample::MultisampleState, rasterization::RasterizationState, vertex_input::{Vertex, VertexDefinition}, viewport::{Viewport, ViewportState}, GraphicsPipelineCreateInfo}, layout::PipelineDescriptorSetLayoutCreateInfo, GraphicsPipeline, Pipeline, PipelineLayout, PipelineShaderStageCreateInfo}, render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass}, shader::ShaderModule, swapchain::Swapchain};

use super::{command::SecondaryCmdBufType, image::VkImage, vertex::{PosInstanceData, PosVertex}, vk::Vk};

/// All the data necessary for constructing a secondary renderpass
pub struct VkSecRenderpass {
    pub cmd_buf: SecondaryCmdBufType,
    pub framebuffer: Arc<Framebuffer>,
    pub rp: Arc<RenderPass>,
    pub clear_values: Vec<Option<ClearValue>>,
}

pub fn descriptor_set(
    vk: Arc<Vk>, 
    set: usize,
    graphics_pipeline: Arc<GraphicsPipeline>,
    writes: impl IntoIterator<Item = WriteDescriptorSet>,
) -> (Arc<PersistentDescriptorSet>, usize) {
    let descriptor_set_allocator = vk.allocators.descriptor_set.clone();
    let pipeline_layout = graphics_pipeline.layout();
    let descriptor_set_layouts = pipeline_layout.set_layouts();

    let descriptor_set_layout_index = set;
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

pub fn framebuffers(
    rp: Arc<RenderPass>, 
    images: &[Arc<Image>]
) -> Vec<Arc<Framebuffer>> {
    images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();
            Framebuffer::new(
                rp.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view],
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect::<Vec<_>>()
}

/* TODO: make attachments an argument to this */
pub fn framebuffers_with_depth(
    vk: Arc<Vk>,
    rp: Arc<RenderPass>, 
    images: &Vec<Arc<Image>>
) -> Vec<Arc<Framebuffer>> {
    let depth_image = ImageView::new_default(
            VkImage::depth(vk.allocators.clone(), Format::D16_UNORM, images[0].extent()).content
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

pub fn render_pass_with_depth(vk: Arc<Vk>, swapchain: Option<Arc<Swapchain>>) -> Arc<RenderPass> {
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
                load_op: Clear, /* Clear */
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

pub fn pipeline(
    vk: Arc<Vk>,
    vs: Arc<ShaderModule>,
    fs: Arc<ShaderModule>,

    render_pass: Arc<RenderPass>,
    viewport: Viewport,
) -> Arc<GraphicsPipeline> {
    let vs = vs.entry_point("main").unwrap();
    let fs = fs.entry_point("main").unwrap();

    let vertex_input_state = PosVertex::per_vertex()
        .definition(&vs.info().input_interface)
        .unwrap();

    let stages = [
        PipelineShaderStageCreateInfo::new(vs),
        PipelineShaderStageCreateInfo::new(fs),
    ];

    let layout = PipelineLayout::new(
        vk.device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
            .into_pipeline_layout_create_info(vk.device.clone())
            .unwrap(),
    )
    .unwrap();

    let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

    GraphicsPipeline::new(
        vk.device.clone(),
        None,
        GraphicsPipelineCreateInfo {
            stages: stages.into_iter().collect(),
            vertex_input_state: Some(vertex_input_state),
            input_assembly_state: Some(InputAssemblyState::default()),
            viewport_state: Some(ViewportState {
                viewports: [viewport].into_iter().collect(),
                ..Default::default()
            }),
            rasterization_state: Some(RasterizationState::default()),
            multisample_state: Some(MultisampleState::default()),
            color_blend_state: Some(ColorBlendState::with_attachment_states(
                subpass.num_color_attachments(),
                ColorBlendAttachmentState::default(),
            )),
            subpass: Some(subpass.into()),
            depth_stencil_state: Some(DepthStencilState::simple_depth_test()),
            ..GraphicsPipelineCreateInfo::layout(layout)
        },
    )
    .unwrap()
}

pub fn instancing_pipeline(
    vk: Arc<Vk>,
    vs: Arc<ShaderModule>,
    fs: Arc<ShaderModule>,

    render_pass: Arc<RenderPass>,
    viewport: Viewport,
) -> Arc<GraphicsPipeline> {
    let vs = vs.entry_point("main").unwrap();
    let fs = fs.entry_point("main").unwrap();

    let vertex_input_state = [PosVertex::per_vertex(), PosInstanceData::per_instance()]
        .definition(&vs.info().input_interface)
        .unwrap();

    let stages = [
        PipelineShaderStageCreateInfo::new(vs),
        PipelineShaderStageCreateInfo::new(fs),
    ];

    let layout = PipelineLayout::new(
        vk.device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
            .into_pipeline_layout_create_info(vk.device.clone())
            .unwrap(),
    )
    .unwrap();

    let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

    GraphicsPipeline::new(
        vk.device.clone(),
        None,
        GraphicsPipelineCreateInfo {
            stages: stages.into_iter().collect(),
            vertex_input_state: Some(vertex_input_state),
            input_assembly_state: Some(InputAssemblyState::default()),
            viewport_state: Some(ViewportState {
                viewports: [viewport].into_iter().collect(),
                ..Default::default()
            }),
            rasterization_state: Some(RasterizationState::default()),
            multisample_state: Some(MultisampleState::default()),
            color_blend_state: Some(ColorBlendState::with_attachment_states(
                subpass.num_color_attachments(),
                ColorBlendAttachmentState::default(),
            )),
            subpass: Some(subpass.into()),
            depth_stencil_state: Some(DepthStencilState::simple_depth_test()),
            ..GraphicsPipelineCreateInfo::layout(layout)
        },
    )
    .unwrap()
}