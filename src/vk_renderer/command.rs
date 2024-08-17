use std::sync::Arc;

use vulkano::{command_buffer::{allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferExecFuture, PrimaryAutoCommandBuffer}, sync::{self, future::{FenceSignalFuture, NowFuture}, GpuFuture}};

use super::Vk;

pub struct VkBuilder(
    pub AutoCommandBufferBuilder<
            PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>, 
            Arc<StandardCommandBufferAllocator>
    >
);

type CommandBufferType = Arc<PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>>;

impl VkBuilder {
    /// Command buffer builder made only for submitting once
    pub fn new_once(vk: &mut Vk) -> Self {
        let builder = AutoCommandBufferBuilder::primary(
            &vk.allocators.command.clone(), 
            vk.queue_family_index, 
            vulkano::command_buffer::CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        Self(builder)
    }

    pub fn command_buffer(self) -> CommandBufferType 
    {
        self.0.build().unwrap()
    }
}

pub fn submit_cmd_buf(vk: &mut Vk, cmd_buf: CommandBufferType) -> FenceSignalFuture<CommandBufferExecFuture<NowFuture>> {
    sync::now(vk.device.clone())
        .then_execute(vk.queue.clone(), cmd_buf)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap()
}