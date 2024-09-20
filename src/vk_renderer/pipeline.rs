use std::sync::Arc;

use vulkano::{descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet}, pipeline::{graphics::viewport::Viewport, layout::PipelineDescriptorSetLayoutCreateInfo, ComputePipeline, GraphicsPipeline, Pipeline, PipelineLayout, PipelineShaderStageCreateInfo}, render_pass::RenderPass, shader::ShaderModule, swapchain::Swapchain};

use super::{command::{submit_cmd_buf, VkBuilder}, shaders::{compute_pipeline, graphics_pipeline}, Vk};

#[derive(Clone)]
pub struct VkGraphicsPipeline {
    pub _vk: Arc<Vk>,
    pub graphics_pipeline: Arc<GraphicsPipeline>,
    pub render_pass: Arc<RenderPass>,
    // pub descriptor_set: Option<Arc<PersistentDescriptorSet>>,
    // pub descriptor_set_layout_index: Option<usize>,
    pub vs: Arc<ShaderModule>,
    pub fs: Arc<ShaderModule>,

    pub pipeline_layout: Arc<dyn Fn() -> Arc<PipelineLayout>>,

    pub viewport: Viewport,
}

impl VkGraphicsPipeline {
    pub fn new(
        vk: Arc<Vk>,
        vs: Arc<vulkano::shader::ShaderModule>,
        fs: Arc<vulkano::shader::ShaderModule>,
        custom_layout: Arc<dyn Fn() -> Arc<PipelineLayout>>,
        swapchain: Option<Arc<Swapchain>>,
    ) -> Self {
        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [1.0, 1.0],
            depth_range: 0.0..=1.0,
        };

        let render_pass = graphics_pipeline::render_pass(
            vk.clone(),
            swapchain
        );

        let graphics_pipeline = graphics_pipeline::graphics_pipeline(
            vk.clone(),
            vs.clone(),
            fs.clone(),
            custom_layout.as_ref(),
            render_pass.clone(),
            viewport.clone(),
        );

        Self {
            _vk: vk,
            graphics_pipeline,
            render_pass,
            viewport,
            pipeline_layout: custom_layout,
            vs,
            fs,
        }
    }

    pub fn default_layout(
        vk: Arc<Vk>,
        vs: Arc<ShaderModule>,
        fs: Arc<ShaderModule>,

    ) -> Arc<dyn Fn() -> Arc<PipelineLayout>> {
        let vs_entry = vs.entry_point("main").unwrap();
        let fs_entry = fs.entry_point("main").unwrap();

        let stages = [
            PipelineShaderStageCreateInfo::new(vs_entry),
            PipelineShaderStageCreateInfo::new(fs_entry),
        ];
        
        let dc_layout = PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages);
        
        Arc::new(move || {
            PipelineLayout::new(
                vk.device.clone(), 
                dc_layout.clone()
                    .into_pipeline_layout_create_info(vk.device.clone())
                    .unwrap(),
            )
            .unwrap()
        })
    }

    pub fn set_descriptor_set_writes(
        &mut self, _writes: impl IntoIterator<Item = WriteDescriptorSet>,
    ) {
        todo!()
    }

    pub fn dispatch(&mut self) {
        todo!()
    }
}


pub struct VkComputePipeline {
    pub vk: Arc<Vk>,
    pub compute_pipeline: Option<Arc<ComputePipeline>>,
    pub descriptor_set: Option<Arc<PersistentDescriptorSet>>,
    pub descriptor_set_layout_index: Option<usize>,
}

impl VkComputePipeline {
    pub fn new(
        vk: Arc<Vk>,
        shader: Arc<vulkano::shader::ShaderModule>,
    ) -> Self {
        let compute_pipeline = compute_pipeline::compute_pipeline(
            vk.clone(),
            shader,
        );

        Self {
            vk,
            compute_pipeline: Some(compute_pipeline),
            descriptor_set: None,
            descriptor_set_layout_index: None,
        }
    }

    pub fn set_descriptor_set_writes(
        &mut self, writes: impl IntoIterator<Item = WriteDescriptorSet>,
    ) {
        let (descriptor_set, dc_layout_idx) = compute_pipeline::descriptor_set(
            self.vk.clone(), 
            self.compute_pipeline.clone().unwrap(),
            writes,
        );

        self.descriptor_set = Some(descriptor_set);
        self.descriptor_set_layout_index = Some(dc_layout_idx);
    }

    pub fn dispatch(&mut self) {
        let mut builder = VkBuilder::new_once(self.vk.clone());
        let workgroup_counts = [1024, 1, 1];

        builder.0
            .bind_pipeline_compute(self.compute_pipeline.clone().unwrap().clone())
            .unwrap()
            .bind_descriptor_sets(
                vulkano::pipeline::PipelineBindPoint::Compute, 
                self.compute_pipeline.clone().unwrap().layout().clone(), 
                self.descriptor_set_layout_index.unwrap() as u32, 
                self.descriptor_set.clone().unwrap(),
            )
            .unwrap()
            .dispatch(workgroup_counts)
            .unwrap();

        let cmd_buf = builder.0.build().unwrap();
        
        let future = submit_cmd_buf(self.vk.clone(), cmd_buf);
                                    // Maybe instead return the future here, instead of submitting it
        future.wait(None).unwrap(); // SUBMITTED ANYWAY!
        
        /*
            TESTING
         */

        // let content = self.data_buffer.content.read().unwrap();
        // for (n, val) in content.iter().enumerate() {
        //     assert_eq!(*val, n as u32 * 12);
        // }

        // println!("Everything succeeded!");
    }
}
