use std::sync::Arc;

use anyhow::{Ok, Result};
use vulkano::{
    VulkanLibrary,
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfo,
        allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo},
    },
    device::{Device, DeviceCreateInfo, QueueCreateFlags, QueueCreateInfo, QueueFlags, physical},
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo},
    memory::{
        self,
        allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    },
    sync::GpuFuture,
};

pub struct GraphicsContext {
    pub instance: Instance,
}
impl GraphicsContext {
    pub fn new() -> () {
        let lib = VulkanLibrary::new().expect("has vulkan");
        let instance = Instance::new(
            lib,
            InstanceCreateInfo {
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                ..Default::default()
            },
        )
        .expect("has vulkan");
        let physical_device = instance
            .enumerate_physical_devices()
            .expect("has vulkan")
            .next()
            .expect("no devices available");
        for family in physical_device.queue_family_properties() {
            println!(
                "found a queue family with {:?} queues(s)",
                family.queue_count
            );
        }
        let queue_family_index = physical_device
            .queue_family_properties()
            .iter()
            .position(|queue_family_properties| {
                queue_family_properties
                    .queue_flags
                    .contains(QueueFlags::GRAPHICS)
            })
            .expect("couldnt find a graphical queue family")
            as u32;
        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
        .expect("has device");
        let queue = queues.next().unwrap();
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let source_content = 0..64;
        let source = Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: (BufferUsage::UNIFORM_BUFFER | BufferUsage::TRANSFER_SRC),
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            source_content,
        )
        .expect("failed to create buffer");
        let destination_content = (0..64).map(|_| 0);
        let destination = Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: (BufferUsage::TRANSFER_DST | BufferUsage::UNIFORM_BUFFER),
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            destination_content,
        )
        .expect("failed to create buffer");

        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default(),
        ));
        let mut builder = AutoCommandBufferBuilder::primary(
            command_buffer_allocator.clone(),
            queue_family_index,
            CommandBufferUsage::OneTimeSubmit,
        )
        .expect("buf");
        builder
            .copy_buffer(CopyBufferInfo::buffers(source.clone(), destination.clone()))
            .unwrap();
        let command_buffer = builder.build().unwrap();
        let orginal = destination.read().unwrap().to_vec();
        let future = vulkano::sync::now(device.clone())
            .then_execute(queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        println!("{:#?}", orginal);
        future.wait(None).unwrap();
        println!("{:#?}", destination.read().unwrap());
    }
}
