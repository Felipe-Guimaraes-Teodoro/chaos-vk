use std::sync::Arc;

use vulkano::{command_buffer::{allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferExecFuture, CommandBufferInheritanceInfo, CommandBufferUsage, PrimaryAutoCommandBuffer, SecondaryAutoCommandBuffer}, sync::{self, future::{FenceSignalFuture, NowFuture}, GpuFuture}};

use super::{renderer::Renderer, Vk};

pub struct VkBuilder(
    pub AutoCommandBufferBuilder<
            PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>, 
            Arc<StandardCommandBufferAllocator>
    >
);

pub type CommandBufferType = Arc<PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>>;
pub type SecondaryCmdBufType = Arc<SecondaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>>;
pub type SecBuilderType = AutoCommandBufferBuilder<SecondaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>, Arc<StandardCommandBufferAllocator>>;
pub type BuilderType = AutoCommandBufferBuilder<PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>, Arc<StandardCommandBufferAllocator>>;

impl VkBuilder {
    /// Command buffer builder made only for submitting once
    pub fn new_once(vk: Arc<Vk>) -> Self {
        let builder = AutoCommandBufferBuilder::primary(
            &vk.allocators.command.clone(), 
            vk.queue_family_index, 
            vulkano::command_buffer::CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        Self(builder)
    }

    /// Command buffer builder made for submitting multiple times
    pub fn new_multiple(vk: Arc<Vk>) -> Self {
        let builder = AutoCommandBufferBuilder::primary(
            &vk.allocators.command.clone(), 
            vk.queue_family_index, 
            vulkano::command_buffer::CommandBufferUsage::MultipleSubmit,
        )
        .unwrap();

        Self(builder)
    }

    pub fn new_secondary(
        vk: Arc<Vk>,
        inheritance_info: Option<CommandBufferInheritanceInfo>,
    ) -> SecBuilderType {
        let builder = AutoCommandBufferBuilder::secondary(
            &vk.allocators.command.clone(),
            vk.queue_family_index,
            CommandBufferUsage::SimultaneousUse,
            match inheritance_info {
                Some(info) => info,
                None => CommandBufferInheritanceInfo::default(),
            }
        )
        .unwrap();
    
        builder
    }

    pub fn command_buffer(self) -> CommandBufferType 
    {
        self.0.build().unwrap()
    }
}

pub fn submit_cmd_buf(vk: Arc<Vk>, cmd_buf: CommandBufferType) -> FenceSignalFuture<CommandBufferExecFuture<NowFuture>> {
    sync::now(vk.device.clone())
        .then_execute(vk.queue.clone(), cmd_buf)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap()
}

impl Renderer {
    
}