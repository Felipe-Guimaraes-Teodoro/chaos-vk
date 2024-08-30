use std::sync::Arc;

use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::CopyBufferInfo;
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use vulkano::pipeline::graphics::vertex_input::Vertex as VulkanoVertex;

use super::command::{submit_cmd_buf, VkBuilder};
use super::vk::MemAllocators;
use super::Vk;

/*
    Example struct to show how data should be handled
    within the context of vulkano buffers
*/
#[derive(BufferContents)]
#[repr(C)]
struct ExampleStruct {
    a: u32,
    b: u32,
}

fn _example_fn(allocators: Arc<MemAllocators>) {
    let example_data = ExampleStruct{a: 0, b: 2};
    let example_buffer = VkBuffer::new(allocators, example_data);

    let _read = example_buffer._content.read().unwrap();
}

pub fn _example_operation(vk: Arc<Vk>) {
    let source_content: Vec<i32> = (0..64).collect();
    let source = Buffer::from_iter(
        vk.allocators.memory.clone(),
        BufferCreateInfo {
            usage: BufferUsage::TRANSFER_SRC,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_HOST
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        source_content,
    )
    .expect("failed to create source buffer");
    
    let destination_content: Vec<i32> = (0..64).map(|_| 0).collect();
    let destination = Buffer::from_iter(
        vk.allocators.memory.clone(),
        BufferCreateInfo {
            usage: BufferUsage::TRANSFER_DST,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_HOST
                | MemoryTypeFilter::HOST_RANDOM_ACCESS,
            ..Default::default()
        },
        destination_content,
    )
    .expect("failed to create destination buffer");

    let mut builder = VkBuilder::new_once(vk.clone());
    builder.0
        .copy_buffer(CopyBufferInfo::buffers(source.clone(), destination.clone()))
        .unwrap();

    let command_buffer = builder.0.build().unwrap();

    let future = submit_cmd_buf(vk, command_buffer);

    future.wait(None).unwrap(); // wait until the GPU has finished the operation

    let src_content = source.read().unwrap();
    let destination_content = destination.read().unwrap();
    assert_eq!(&*src_content, &*destination_content);

    println!("Example operation succeded!")
}

#[derive(Clone)]
pub struct VkBuffer<T: BufferContents> {
    pub _content: Subbuffer<T>,
}

impl<T: BufferContents> VkBuffer<T> {
    pub fn new(allocators: Arc<MemAllocators>, data: T) -> Self {
        let buffer = Buffer::from_data(
            allocators.memory.clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE, // MemoryTypeFilter::HOST_RANDOM_ACCESS is more suitable if data is being written continuously to this buffer
                ..Default::default()
            },
            data,
        )
        .expect("failed to create buffer");

        Self {
            _content: buffer,
        }
    }
}

#[derive(Clone)]
pub struct VkIterBuffer<T: BufferContents> {
    pub content: Subbuffer<[T]>,
}

impl<T: BufferContents> VkIterBuffer<T> {
    pub fn uniform<I>(allocators: Arc<MemAllocators>, iter_data: I) -> Self 
    where 
        T: BufferContents,
        I: Iterator<Item = T> + ExactSizeIterator
    {
        let buffer = Buffer::from_iter(
            allocators.memory.clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            iter_data,
        )
        .expect("failed to create buffer");

        Self {
            content: buffer,
        }
    }

    pub fn storage<I>(allocators: Arc<MemAllocators>, iter_data: I) -> Self 
    where 
        T: BufferContents,
        I: Iterator<Item = T> + ExactSizeIterator
    {
        let buffer = Buffer::from_iter(
            allocators.memory.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            iter_data,
        )
        .expect("failed to create buffer");

        Self {
            content: buffer,
        }
    }

    pub fn transfer_dst<I>(allocators: Arc<MemAllocators>, iter_data: I) -> Self 
    where 
        T: BufferContents,
        I: Iterator<Item = T> + ExactSizeIterator
    {
        let buffer = Buffer::from_iter(
            allocators.memory.clone(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_RANDOM_ACCESS,
                ..Default::default()
            },
            iter_data,
        )
        .expect("failed to create buffer");

        Self {
            content: buffer,
        }
    }

    pub fn vertex(allocators: Arc<MemAllocators>, vertices: Vec<T>) -> Self 
    where 
        T: BufferContents + VulkanoVertex
    {
        let buffer = Buffer::from_iter(
            allocators.memory.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices,
        )
        .expect("failed to create buffer");

        Self {
            content: buffer,
        }
    }

    pub fn index(allocators: Arc<MemAllocators>, vertices: Vec<T>) -> Self 
    where 
        T: BufferContents 
    {
        let buffer = Buffer::from_iter(
            allocators.memory.clone(),
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices,
        )
        .expect("failed to create buffer");

        Self {
            content: buffer,
        }
    }
}