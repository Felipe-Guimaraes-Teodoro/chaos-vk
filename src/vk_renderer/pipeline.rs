use std::sync::Arc;

use vulkano::{descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet}, pipeline::{ComputePipeline, Pipeline}};

use super::{command::{submit_cmd_buf, VkBuilder}, shaders::{compute_shader, mandelbrot_shader, pipeline_shader}, Vk};

pub struct VkGraphicsPipeline {
    pub vk: Arc<Vk>,
    pub compute_pipeline: f32,
    pub descriptor_set: Option<Arc<PersistentDescriptorSet>>,
    pub descriptor_set_layout_index: Option<usize>,
}

impl VkGraphicsPipeline {
    pub fn new(
        vk: Arc<Vk>,
        vs: Arc<vulkano::shader::ShaderModule>,
        fs: Arc<vulkano::shader::ShaderModule>,
    ) -> Self {
        let graphics_pipeline = pipeline_shader::graphics_pipeline(
            vk.clone(),
            vs,
            fs
        );

        todo!();
    }

    pub fn set_descriptor_set_writes(
        &mut self, writes: impl IntoIterator<Item = WriteDescriptorSet>,
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
        let compute_pipeline = compute_shader::compute_pipeline(
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
        let (descriptor_set, dc_layout_idx) = compute_shader::descriptor_set(
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
