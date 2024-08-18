use std::sync::Arc;

use vulkano::{buffer::BufferContents, descriptor_set::PersistentDescriptorSet, pipeline::{ComputePipeline, Pipeline}};

use super::{buffer::VkIterBuffer, command::{submit_cmd_buf, VkBuilder}, shaders::compute_shaders, Vk};


/*
    CURRENTLY SET AS AN EXAMPLE COMPUTE PIPELINE
*/
pub struct VkComputePipeline<I: BufferContents> {
    pub vk: Arc<Vk>,
    pub data_buffer: VkIterBuffer<I>,
    pub compute_pipeline: Option<Arc<ComputePipeline>>,
    pub descriptor_set: Option<Arc<PersistentDescriptorSet>>,
    pub descriptor_set_layout_index: Option<usize>,
}

impl VkComputePipeline<u32> {
    pub fn new(
        vk: Arc<Vk>,
        data_buffer: VkIterBuffer<u32>,
    ) -> Self {
        let compute_pipeline = compute_shaders::compute_pipeline(vk.clone());
        let (descriptor_set, dc_layout_idx) = compute_shaders::descriptor_set(
            vk.clone(), 
            compute_pipeline.clone(),
            data_buffer.content.clone()
        );

        Self {
            vk,
            data_buffer,
            compute_pipeline: Some(compute_pipeline),
            descriptor_set: Some(descriptor_set),
            descriptor_set_layout_index: Some(dc_layout_idx),
        }
    }

    pub fn dispatch(&mut self) {
        // let command_buffer_allocator = self.vk.allocators.command.clone();
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
        future.wait(None).unwrap();

        /*
            TESTING
         */
        let content = self.data_buffer.content.read().unwrap();
        for (n, val) in content.iter().enumerate() {
            assert_eq!(*val, n as u32 * 12);
        }

        println!("Everything succeeded!");
    }
}