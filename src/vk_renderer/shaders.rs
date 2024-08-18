use std::sync::Arc;

use vulkano::{pipeline::{compute::ComputePipelineCreateInfo, layout::{PipelineDescriptorSetLayoutCreateInfo, PipelineLayoutCreateInfo}, ComputePipeline, PipelineLayout, PipelineShaderStageCreateInfo}, shader::ShaderModule};

use super::Vk;

pub mod compute_shaders {
    use std::sync::Arc;

    use vulkano::{buffer::{BufferContents, Subbuffer}, descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet}, pipeline::{compute::ComputePipelineCreateInfo, layout::PipelineDescriptorSetLayoutCreateInfo, ComputePipeline, Pipeline, PipelineLayout, PipelineShaderStageCreateInfo}};

    use crate::{buffer::VkIterBuffer, Vk};

    vulkano_shaders::shader!{
        ty: "compute",
        src: r"
            #version 460

            layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

            layout(set = 0, binding = 0) buffer Data {
                uint data[];
            } buf;

            void main() {
                uint idx = gl_GlobalInvocationID.x;
                buf.data[idx] *= 12;
            }
        ",
    }

    pub fn compute_pipeline(vk: Arc<Vk>) -> Arc<ComputePipeline> {
        let shader = load(vk.device.clone())
        .expect("failed to create shader module");

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

    pub fn descriptor_set<I: BufferContents>(
        vk: Arc<Vk>, 
        compute_pipeline: Arc<ComputePipeline>,
        content: Subbuffer<[I]>,
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
            [WriteDescriptorSet::buffer(0, content)], // 0 is the binding
            [],
        )
        .unwrap(), descriptor_set_layout_index)
    }
}
