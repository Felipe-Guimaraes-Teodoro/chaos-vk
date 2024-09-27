/* !!!WORK IN PROGRESS!!! */
#![allow(unused_imports, dead_code, deprecated)]

use ahash::HashSetExt;
use vulkano::{buffer::BufferUsage, command_buffer::{allocator::StandardCommandBufferAllocator, CopyBufferToImageInfo, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents}, descriptor_set::{DescriptorImageViewInfo, WriteDescriptorSet, WriteDescriptorSetElements}, image::{sampler::Sampler, view::{ImageView, ImageViewCreateInfo, ImageViewType}, Image, ImageCreateInfo, ImageType, ImageUsage}, memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter}, pipeline::{graphics::{color_blend::ColorBlendState, input_assembly::InputAssemblyState, multisample::MultisampleState, rasterization::RasterizationState, vertex_input::{Vertex, VertexDefinition, VertexInputState}, viewport::{Scissor, Viewport, ViewportState}, GraphicsPipelineCreateInfo}, layout::PipelineDescriptorSetLayoutCreateInfo, DynamicState, Pipeline, PipelineLayout, PipelineShaderStageCreateInfo}, render_pass::RenderPass, shader::ShaderStages};
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::device::{Device, Queue};
use vulkano::pipeline::GraphicsPipeline;
use vulkano::sync::GpuFuture;

// use vulkano::sampler::{Sampler, SamplerAddressMode, Filter, MipmapMode};
use vulkano::format::{Format, ClearValue};
use vulkano::render_pass::Subpass;
use vulkano::render_pass::Framebuffer;

use std::{collections::HashSet, sync::Arc};
use std::fmt;

use imgui::{DrawVert, Textures, DrawCmd, DrawCmdParams, internal::RawWrapper, TextureId, ImString};
use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex as VulkanoVertex};

use crate::command::BuilderType;

use super::super::{command::{VkBuilder, submit_cmd_buf}, Vk, buffer::VkIterBuffer, shaders::graphics_pipeline::descriptor_set};

#[derive(Default, Debug, Clone, VulkanoVertex, BufferContents)]
#[repr(C)]
struct ImVertex {
    #[format(R32G32_SFLOAT)]
    pub pos: [f32; 2],
    #[format(R32G32_SFLOAT)]
    pub uv : [f32; 2],
    //#[format(R8G8B8A8_UNORM)]
    //pub col: [u8; 4],

    #[format(R32_UINT)]
    pub col: u32, /* packed color */
}

#[derive(Debug)]
pub enum RendererError {
    BadTexture(TextureId),
    BadImageDimensions(),
}

pub struct ImRenderer {
    render_pass: Arc<RenderPass>,
    pipeline: Arc<GraphicsPipeline>,
    font_texture: Arc<Image>,
    textures: Textures<Image>,
    vrt_buffer_pool: VkIterBuffer<ImVertex>,
    idx_buffer_pool: VkIterBuffer<u16>,
}

impl ImRenderer {
    pub fn new(ctx: &mut imgui::Context, vk: Arc<Vk>, format : Format) -> Result<ImRenderer, Box<dyn std::error::Error>> {
        let vs = super::shaders::imvs::load(vk.device.clone()).unwrap();
        let fs = super::shaders::imfs::load(vk.device.clone()).unwrap();
        
        let vs_entry = vs.entry_point("main").unwrap();
        let fs_entry = fs.entry_point("main").unwrap();

        let vertex_input_state = ImVertex::per_vertex()
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
        
        let render_pass = vulkano::single_pass_renderpass!(vk.device.clone(),
                attachments: {
                    color_attachment: {
                        format: format,
                        samples: 1,
                        load_op: Load,
                        store_op: Store,
                    },
                },
                pass: {
                    color: [color_attachment],
                    depth_stencil: {},
                },
            )
        .unwrap();
        
    
        let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

        let mut dyn_state = ahash::HashSet::new();
        dyn_state.insert(DynamicState::ViewportWithCount);
        dyn_state.insert(DynamicState::ScissorWithCount);

        let pipeline = GraphicsPipeline::new(
            vk.device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),

                vertex_input_state: Some(vertex_input_state),
                
                input_assembly_state: Some(InputAssemblyState::new().topology(vulkano::pipeline::graphics::input_assembly::PrimitiveTopology::TriangleList)),
        
                viewport_state: Some(ViewportState::new()),

                rasterization_state: Some(RasterizationState::default()),

                multisample_state: Some(MultisampleState::default()),
                
                color_blend_state: Some(ColorBlendState::new(1).blend_alpha()),

                subpass: Some(vulkano::pipeline::graphics::subpass::PipelineSubpassType::BeginRenderPass(subpass.clone())),

                dynamic_state: dyn_state,

                ..GraphicsPipelineCreateInfo::layout(layout)
            }
        )
        .unwrap();

        let textures = Textures::new();

        let font_texture = Self::upload_font_texture(ctx.fonts().build_rgba32_texture(), vk.allocators.memory.clone(), vk.clone());

        // ctx.set_renderer_name(Some(ImString::from(format!("imgui-vulkano-renderer {}", env!("CARGO_PKG_VERSION")))));
        let vrt_buffer_pool = VkIterBuffer::vertex(vk.allocators.clone(), vec![ImVertex { 
            pos: [0.0, 0.0], uv: [0.0, 0.0], col: 0, 
        }]);
        let idx_buffer_pool = VkIterBuffer::index(vk.allocators.clone(), vec![0]);

        Ok(ImRenderer {
            render_pass,
            pipeline,
            font_texture,
            textures,
            vrt_buffer_pool,
            idx_buffer_pool,
        })
    }

    pub fn draw_commands(&mut self, cmd_buf_builder: &mut BuilderType, framebuffer: Arc<Framebuffer>, draw_data: &imgui::DrawData, vk: Arc<Vk>) {
        let fb_width = draw_data.display_size[0] * draw_data.framebuffer_scale[0];
        let fb_height = draw_data.display_size[1] * draw_data.framebuffer_scale[1];
        if !(fb_width > 0.0 && fb_height > 0.0) {
            return;
        }

        let left = draw_data.display_pos[0];
        let right = draw_data.display_pos[0] + draw_data.display_size[0];
        let top = draw_data.display_pos[1];
        let bottom = draw_data.display_pos[1] + draw_data.display_size[1];

        let _pc = super::shaders::imvs::VertPC {
            matrix : [
                [(2.0 / (right - left)), 0.0, 0.0, 0.0],
                [0.0, (2.0 / (bottom - top)), 0.0, 0.0],
                [0.0, 0.0, -1.0, 0.0],
                [
                    (right + left) / (left - right),
                    (top + bottom) / (top - bottom),
                    0.0,
                    1.0,
                ],
            ]
        };

        let _dims = framebuffer.attachments()[0].image().extent();

        let clip_off = draw_data.display_pos;
        let clip_scale = draw_data.framebuffer_scale;

        cmd_buf_builder
            .begin_render_pass(
                RenderPassBeginInfo::framebuffer(framebuffer),
                SubpassBeginInfo {
                    contents: SubpassContents::Inline,
                    ..Default::default()
                }
            )
            .unwrap();

        for draw_list in draw_data.draw_lists() {
            for cmd in draw_list.commands() {
                match cmd {
                    DrawCmd::Elements { 
                        count, 
                        cmd_params: DrawCmdParams {
                            clip_rect,
                            texture_id,
                            vtx_offset,
                            idx_offset,
                        }
                    } => {
                        // let clip_rect = [
                        //     (clip_rect[0] - clip_off[0]) * clip_scale[0],
                        //     (clip_rect[1] - clip_off[1]) * clip_scale[1],
                        //     (clip_rect[2] - clip_off[0]) * clip_scale[0],
                        //     (clip_rect[3] - clip_off[1]) * clip_scale[1],
                        // ];

                        let tex = self.lookup_texture(texture_id);

                        let set = descriptor_set(
                            vk.clone(), 
                            0, 
                            self.pipeline.clone(), 
                            [WriteDescriptorSet::image_view(0, tex.unwrap())],
                        );

                        cmd_buf_builder
                            .bind_pipeline_graphics(self.pipeline.clone())
                            .unwrap()
                            .bind_descriptor_sets(
                                vulkano::pipeline::PipelineBindPoint::Graphics, 
                                self.pipeline.layout().clone(), 
                                0, 
                                set.0,
                            )
                            .unwrap()
                            .bind_vertex_buffers(0, self.vrt_buffer_pool.content.clone())
                            .unwrap()
                            .bind_index_buffer(
                                self.idx_buffer_pool.content.clone()
                            )
                            .unwrap()
                            .draw_indexed(
                                count as u32, 
                                1, 
                                0, 
                                0, 
                                0
                            )
                            .unwrap();

                    },
                    DrawCmd::ResetRenderState => {
                        ()
                    },
                    DrawCmd::RawCallback { callback, raw_cmd } => unsafe {
                        callback(draw_list.raw(), raw_cmd);
                    },
                }

            }
        } /* for draw list in ... */


    }

    pub fn reload_font_texture(
        &mut self,
        ctx: &mut imgui::Context,
        _device : Arc<Device>,
        vk: Arc<Vk>,
        _queue : Arc<Queue>,
    ) {
        let upload_font_texture = Self::upload_font_texture(ctx.fonts().build_rgba32_texture(), vk.allocators.memory.clone(), vk.clone()); 
        self.font_texture = upload_font_texture;
    }
    
    pub fn textures(&mut self) -> &mut Textures<Image> {
        &mut self.textures
    }

    fn upload_font_texture(
        fonts: imgui::FontAtlasTexture,
        allocator: Arc<dyn MemoryAllocator>,
        vk: Arc<Vk>,
    ) -> Arc<Image> {
        let image = Image::new(
            allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::R8G8B8A8_SRGB,
                extent: [fonts.width, fonts.height, 1],
                usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST,
                ..Default::default()
            },
        )
        .unwrap();

        let mut builder = VkBuilder::new_once(vk.clone());

        let buffer = VkIterBuffer::transfer_src(
            vk.allocators.clone(), 
            (0..(fonts.width*fonts.height*4)).map(|i| {
                fonts.data[i as usize]
            }),
        );

        builder.0
            .copy_buffer_to_image(
                CopyBufferToImageInfo::buffer_image(buffer.content.clone(), image.clone())
            )
            .unwrap();

        let cmd_buf = builder.command_buffer();
        
        let fut = submit_cmd_buf(vk.clone(), cmd_buf);
        fut.wait(None).unwrap();

        image
    }

    fn lookup_texture(&self, texture_id: TextureId) -> Option<Arc<ImageView>> {
        if texture_id.id() == usize::MAX {
            return Some(ImageView::new(
                self.font_texture.clone(), 
                ImageViewCreateInfo { 
                    format: Format::R8G8B8A8_SRGB, 
                    usage: ImageUsage::STORAGE,
                    ..Default::default()
                }
            ).unwrap());
        } else {
            None
        }
    }
}
