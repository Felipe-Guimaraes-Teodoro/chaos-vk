/* 

/* !!!WORK IN PROGRESS!!! */

use vulkano::{buffer::BufferUsage, command_buffer::{CopyBufferToImageInfo, PrimaryAutoCommandBuffer, SubpassContents}, image::{sampler::Sampler, view::{ImageView, ImageViewCreateInfo, ImageViewType}, Image, ImageCreateInfo, ImageType, ImageUsage}, memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter}, pipeline::{graphics::{color_blend::ColorBlendState, input_assembly::InputAssemblyState, vertex_input::{Vertex, VertexDefinition, VertexInputState}, viewport::{Scissor, Viewport, ViewportState}, GraphicsPipelineCreateInfo}, layout::PipelineDescriptorSetLayoutCreateInfo, DynamicState, Pipeline, PipelineLayout, PipelineShaderStageCreateInfo}, render_pass::RenderPass, shader::ShaderStages};
use vulkano::command_buffer::{AutoCommandBufferBuilder};
use vulkano::device::{Device, Queue};
use vulkano::pipeline::{GraphicsPipeline};
use vulkano::sync::GpuFuture;

// use vulkano::sampler::{Sampler, SamplerAddressMode, Filter, MipmapMode};
use vulkano::format::{Format, ClearValue};
use vulkano::render_pass::Subpass;
use vulkano::render_pass::Framebuffer;

use std::sync::Arc;
use std::fmt;

use imgui::{DrawVert, Textures, DrawCmd, DrawCmdParams, internal::RawWrapper, TextureId, ImString};
use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex as VulkanoVertex};

use crate::{buffer::{VkBuffer, VkIterBuffer}, command::{submit_cmd_buf, VkBuilder}, pipeline::VkGraphicsPipeline, Vk};

#[derive(Default, Debug, Clone, VulkanoVertex, BufferContents)]
#[repr(C)]
struct ImVertex {
    #[format(R32G32_SFLOAT)]
    pub pos: [f32; 2],
    #[format(R32G32_SFLOAT)]
    pub uv : [f32; 2],
    #[format(R8G8B8A8_UNORM)]
    pub col: u32, /* packed color */
    // pub col: [u8; 4],
}

#[derive(Debug)]
pub enum RendererError {
    BadTexture(TextureId),
    BadImageDimensions(),
}

pub struct Renderer {
    render_pass : Arc<RenderPass>,
    pipeline : Arc<GraphicsPipeline>,
    font_texture : Arc<Image>,
    textures : Textures<Image>,
    vrt_buffer_pool : VkIterBuffer<ImVertex>,
    idx_buffer_pool : VkIterBuffer<u16>,
}

impl Renderer {
    pub fn init(ctx: &mut imgui::Context, vk: Arc<Vk>, format : Format) -> Result<Renderer, Box<dyn std::error::Error>> {
        let vs = super::shaders::vs::load(vk.device.clone()).unwrap();
        let fs = super::shaders::fs::load(vk.device.clone()).unwrap();
        

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

        let pipeline = GraphicsPipeline::new(
            vk.device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                vertex_input_state: Some(vertex_input_state),
                
                input_assembly_state: Some(InputAssemblyState::new().topology(vulkano::pipeline::graphics::input_assembly::PrimitiveTopology::TriangleList)),
        
                viewport_state: Some(ViewportState::new()),
                
                color_blend_state: Some(ColorBlendState::new(1).blend_alpha()),

                subpass: Some(vulkano::pipeline::graphics::subpass::PipelineSubpassType::BeginRenderPass(subpass.clone())),

                ..GraphicsPipelineCreateInfo::layout(layout)
            }
        )
        .unwrap();

        let textures = Textures::new();

        let font_texture = Self::upload_font_texture(ctx.fonts(), vk.device.clone(), vk.queue.clone());

        // ctx.set_renderer_name(Some(ImString::from(format!("imgui-vulkano-renderer {}", env!("CARGO_PKG_VERSION")))));

        let vrt_buffer_pool = VkIterBuffer::vertex(vk.allocators.clone(), vec![]);
        let idx_buffer_pool = VkIterBuffer::index(vk.allocators.clone(), vec![]);

        Ok(Renderer {
            render_pass,
            pipeline,
            font_texture,
            textures,
            vrt_buffer_pool,
            idx_buffer_pool,
        })
    }

    pub fn draw_commands<I>(&mut self, cmd_buf_builder : &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, _queue : Arc<Queue>, target : ImageView, draw_data : &imgui::DrawData) -> Result<(), Box<dyn std::error::Error>> {

        let fb_width = draw_data.display_size[0] * draw_data.framebuffer_scale[0];
        let fb_height = draw_data.display_size[1] * draw_data.framebuffer_scale[1];
        if !(fb_width > 0.0 && fb_height > 0.0) {
            return Ok(());
        }
        let left = draw_data.display_pos[0];
        let right = draw_data.display_pos[0] + draw_data.display_size[0];
        let top = draw_data.display_pos[1];
        let bottom = draw_data.display_pos[1] + draw_data.display_size[1];

        let pc = super::shaders::vs::VertPC {
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

        let dims = target.image().extent();

        Ok(())
    }

    pub fn reload_font_texture(
        &mut self,
        ctx: &mut imgui::Context,
        device : Arc<Device>,
        queue : Arc<Queue>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // self.font_texture = Self::upload_font_texture(ctx.fonts(), device, queue)?;
        Ok(())
    }
    
    pub fn textures(&mut self) -> &mut Textures<Image> {
        &mut self.textures
    }

    fn upload_font_texture(
        mut fonts: imgui::FontAtlasTexture,
        allocator: Arc<dyn MemoryAllocator>,
        vk: Arc<Vk>,
        device : Arc<Device>,
        queue : Arc<Queue>,
    ) -> Arc<Image> 
    {
        let texture = fonts.data;

        let image = Image::new(
            allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::R8G8B8A8_SRGB,
                extent: [1024, 1024, 1],
                usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST,
                ..Default::default()
            },
        )
        .unwrap();

        let mut builder = VkBuilder::new_once(vk.clone());

        let buffer = VkIterBuffer::transfer_dst(
            vk.allocators.clone(), 
            (0..fonts.width*fonts.height*4).map(|i| {
                fonts.data[i as usize]
            }),
        );

        builder.0
            .copy_buffer_to_image(
                CopyBufferToImageInfo::buffer_image(buffer.content, image.clone())
            );

        let cmd_buf = builder.command_buffer();
        
        let fut = submit_cmd_buf(vk.clone(), cmd_buf);
        fut.wait(None).unwrap();

        image
    }

    fn lookup_texture(&self, texture_id: TextureId) -> Result<Arc<Image>, RendererError> {
        if texture_id.id() == usize::MAX {
            Ok(&self.font_texture)
        } else if let Some(texture) = self.textures.get(texture_id) {
            Ok(texture)
        } else {
            Err(RendererError::BadTexture(texture_id))
        }
    }
}

*/