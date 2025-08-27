use std::sync::Arc;

use bevy::{ecs::system::Resource, log::info_span};
use biosim_core::{world::Cell, WORLD_WIDTH};
use vulkano::{buffer::{BufferUsage, CpuAccessibleBuffer}, command_buffer::{allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage}, descriptor_set::{allocator::StandardDescriptorSetAllocator, layout::DescriptorType, PersistentDescriptorSet, WriteDescriptorSet}, device::{Device, DeviceCreateInfo, DeviceExtensions, Features, Queue, QueueCreateInfo, QueueFlags}, instance::{Instance, InstanceCreateInfo}, memory::allocator::StandardMemoryAllocator, pipeline::{ComputePipeline, Pipeline, PipelineBindPoint}, shader::{spirv::{Capability, ExecutionModel}, DescriptorRequirements, EntryPointInfo, ShaderExecution, ShaderInterface, ShaderModule, ShaderStages}, sync::{self, GpuFuture}, Version, VulkanLibrary};

#[derive(Resource)]
pub struct BiosimComputeShader {
    device: Arc<Device>,
    queue: Arc<Queue>,
    pipeline: Arc<ComputePipeline>,
    descriptor_set: Arc<PersistentDescriptorSet>,
    buffer_allocator: StandardCommandBufferAllocator,
    input_buffer: Arc<CpuAccessibleBuffer<[Cell]>>, // Note that this means we have a fixed sized for our buffer. If we want variable size, we'd need to rebuild the buffer each time.
    output_buffer: Arc<CpuAccessibleBuffer<[Cell]>>,
}

impl BiosimComputeShader {
    pub fn dispatch(&self, input: &Vec<Cell>) -> Vec<Cell> {
        let copy_span = info_span!("copying").entered();
        self.copy_to_buffer(input);
        copy_span.exit();

        let mut builder = AutoCommandBufferBuilder::primary(
                &self.buffer_allocator,
                self.queue.queue_family_index(),
                CommandBufferUsage::OneTimeSubmit,
            ).unwrap();
        builder.bind_pipeline_compute(self.pipeline.clone())
            .bind_descriptor_sets(PipelineBindPoint::Compute, self.pipeline.layout().clone(), 0, self.descriptor_set.clone())
            .dispatch([WORLD_WIDTH as u32, WORLD_WIDTH as u32, 1])
            .unwrap();

        let command_buffer = builder.build().unwrap();

        let gpu_execution_span = info_span!("gpu").entered();
        let future = sync::now(self.device.clone())
            .then_execute(self.queue.clone(), command_buffer).unwrap()
            .then_signal_fence_and_flush().unwrap();
        future.wait(None).unwrap();
        gpu_execution_span.exit();

        let read_back_span = info_span!("read_back").entered();
        let content = self.output_buffer.read().unwrap().to_vec();
        read_back_span.exit();
        content
    }

    fn copy_to_buffer(&self, input: &[Cell]) {
        let mut content = self.input_buffer.write().unwrap();
        for (src, dst) in input.iter().zip(content.iter_mut()) {
            *dst = *src;
        }
    }

    pub fn new(buffer_length: usize) -> BiosimComputeShader {
        let library = VulkanLibrary::new().unwrap();
        let instance = Instance::new(library, InstanceCreateInfo {
            ..Default::default()
        }).unwrap();

        let physical_devices = instance.enumerate_physical_devices().unwrap();

        let (physical_device, queue_family_index) = physical_devices
            .filter_map(|pdev| {
                pdev.queue_family_properties()
                    .iter()
                    .enumerate()
                    .find(|(_, q)| q.queue_flags.intersects(&QueueFlags { compute: true, ..Default::default() }))
                    .map(|(index, _)| (pdev.clone(), index as u32))
            })
            .next()
            .expect("No device with compute capability found");

        println!("Selected device: {}", physical_device.properties().device_name);
        
        let supported_features = physical_device.supported_features();
        if !supported_features.vulkan_memory_model {
                panic!("Selected physical device does not support vulkan_memory_model feature required by the shader");
        }

        let features = Features {
            vulkan_memory_model: true,
            ..Features::default()
        };

        let device_extensions = DeviceExtensions {
            khr_vulkan_memory_model: true,
            ..Default::default()
        };

        let (device, mut queues) = Device::new(
                physical_device,
                DeviceCreateInfo {
                    enabled_features: features,
                    enabled_extensions: device_extensions,
                    queue_create_infos: vec![QueueCreateInfo {
                        queue_family_index,
                        ..Default::default()
                    }],
                    ..Default::default()
                }
            )
            .unwrap();

        let queue = queues.next().unwrap();

        let spirv_bytes = std::fs::read(env!("biosim_rust_shader.spv")).unwrap();
        // For some reason, when I just use from_bytes here, we end up with the equivalent of the
        // `descriptor_requirements` being `[]`, which causes a segfault a bit later on. I'm not
        // sure at the moment if this is an issue with vulkano or rust-gpu, or something I'm doing
        // wrong. However, by mimicking the structure of the `EntryPointInfo` generated when we use
        // `vulkano_shaders:shader!` with a `src` property, we can get the compute shader to load
        // correctly.
        let shader = unsafe { ShaderModule::from_bytes_with_data(
            device.clone(),
            &spirv_bytes, 
            Version::major_minor(1, 3), 
            [&Capability::Shader, &Capability::VulkanMemoryModel], 
            [], 
            [(
                "main".to_owned(),
                ExecutionModel::GLCompute,
                EntryPointInfo {
                    execution: ShaderExecution::Compute, 
                    descriptor_requirements: [
                        (
                            (0u32, 0u32), 
                            DescriptorRequirements {
                                descriptor_types: vec![DescriptorType:: StorageBuffer, DescriptorType :: StorageBufferDynamic],
                                descriptor_count: Some(1u32),
                                stages: ShaderStages { compute: true, ..Default::default() },
                                storage_write: [0u32].into_iter().collect(),
                                ..Default::default()
                            }
                        ),
                        (
                            (0u32, 1u32), 
                            DescriptorRequirements {
                                descriptor_types: vec![DescriptorType:: StorageBuffer, DescriptorType :: StorageBufferDynamic],
                                descriptor_count: Some(1u32),
                                stages: ShaderStages { compute: true, ..Default::default() },
                                storage_write: [1u32].into_iter().collect(),
                                ..Default::default()
                            }
                        ),
                    ].into_iter().collect(),
                    push_constant_requirements: None,
                    specialization_constant_requirements: [].into_iter().collect(),
                    input_interface: ShaderInterface::new_unchecked(vec! []),
                    output_interface: ShaderInterface::new_unchecked(vec! []),
                }
            )])}.unwrap();

        let memory_allocator = StandardMemoryAllocator::new_default(device.clone());
        let input_buffer = CpuAccessibleBuffer::from_iter(
            &memory_allocator, 
            BufferUsage { storage_buffer: true, ..Default::default() }, 
            false, 
            vec![Cell::Dead; buffer_length],
        ).unwrap();
        let output_buffer = CpuAccessibleBuffer::from_iter(
            &memory_allocator, 
            BufferUsage { storage_buffer: true, ..Default::default() }, 
            false, 
            vec![Cell::Dead; buffer_length]
        ).unwrap();

        let pipeline = ComputePipeline::new(device.clone(), shader.entry_point("main").unwrap(), &(), None, |_| {}).unwrap();
        let layout = pipeline.layout().set_layouts().get(0).unwrap();
        let descriptor_set_allocator = StandardDescriptorSetAllocator::new(device.clone());
        let descriptor_set = PersistentDescriptorSet::new(
            &descriptor_set_allocator,
            layout.clone(),
            [WriteDescriptorSet::buffer(0, input_buffer.clone()), WriteDescriptorSet::buffer(1, output_buffer.clone())],
        ).unwrap();

        let buffer_allocator = StandardCommandBufferAllocator::new(device.clone(), Default::default());
        BiosimComputeShader { device, queue, pipeline, descriptor_set, buffer_allocator, input_buffer, output_buffer }
    }
}