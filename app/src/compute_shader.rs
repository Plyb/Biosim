use std::{mem, num::NonZero, sync::{mpsc::channel, Arc}};

use bevy::{ecs::system::{IntoSystem, Resource}, log::info_span, render::{render_resource::{Buffer, ComputePipeline}, renderer::{RenderDevice, RenderQueue}}};
use biosim_core::{world::Cell, WORLD_WIDTH};
use ndarray::{ArrayBase, ArrayView, Dim, OwnedRepr, SliceArg};
use pollster::FutureExt;
use wgpu::{util::BufferInitDescriptor, BindGroup, BindGroupLayoutEntry, BufferDescriptor, DeviceDescriptor, Features, InstanceDescriptor, Limits, PipelineLayout, PipelineLayoutDescriptor, ShaderStages};
// use vulkano::{buffer::{BufferUsage, CpuAccessibleBuffer}, command_buffer::{allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage}, descriptor_set::{allocator::StandardDescriptorSetAllocator, layout::{DescriptorSetLayout, DescriptorType}, PersistentDescriptorSet, WriteDescriptorSet}, device::{Device, DeviceCreateInfo, DeviceExtensions, Features, Queue, QueueCreateInfo, QueueFlags}, instance::{Instance, InstanceCreateInfo}, memory::allocator::StandardMemoryAllocator, pipeline::{ComputePipeline, Pipeline, PipelineBindPoint}, shader::{spirv::{Capability, ExecutionModel}, DescriptorRequirements, EntryPointInfo, ShaderExecution, ShaderInterface, ShaderModule, ShaderStages}, sync::{self, GpuFuture}, Version, VulkanLibrary};

#[derive(Resource)]
pub struct BiosimComputeShader {
    render_device: RenderDevice,
    render_queue: RenderQueue,
    // device: Arc<Device>,
    // queue: Queue,
    pipeline: ComputePipeline,
    bind_group: BindGroup,
    // descriptor_set_allocator: StandardDescriptorSetAllocator,
    // descriptor_set: Arc<PersistentDescriptorSet>,
    // layout: Arc<DescriptorSetLayout>,
    // buffer_allocator: StandardCommandBufferAllocator,
    // input_buffer: Arc<CpuAccessibleBuffer<[Cell]>>, // Note that this means we have a fixed sized for our buffer. If we want variable size, we'd need to rebuild the buffer each time.
    staging_input_buffer: Buffer,
    pub input_buffer: Buffer,
    output_buffer: Buffer,
    staging_output_buffer: Buffer,
}

impl BiosimComputeShader {
    pub fn dispatch(&self) {
        const THREADS_PER_WORKGROUP: u32 = 32;

        // let mut builder = AutoCommandBufferBuilder::primary(
        //         &self.buffer_allocator,
        //         self.queue.queue_family_index(),
        //         CommandBufferUsage::OneTimeSubmit,
        //     ).unwrap();
        // builder.bind_pipeline_compute(self.pipeline.clone())
        //     .bind_descriptor_sets(PipelineBindPoint::Compute, self.pipeline.layout().clone(), 0, self.descriptor_set.clone())
        //     .dispatch([WORLD_WIDTH as u32 / THREADS_PER_WORKGROUP, WORLD_WIDTH as u32 / THREADS_PER_WORKGROUP, 1])
        //     .unwrap();
        let mut encoder = self.render_device.create_command_encoder(&Default::default());

        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.dispatch_workgroups(WORLD_WIDTH as u32 / THREADS_PER_WORKGROUP, WORLD_WIDTH as u32 / THREADS_PER_WORKGROUP, 1);
        }

        // let command_buffer = builder.build().unwrap();

        self.render_queue.submit([encoder.finish()]);
        
        let gpu_execution_span = info_span!("gpu").entered();
        // let future = sync::now(self.device.clone())
        //     .then_execute(self.queue.clone(), command_buffer).unwrap()
        //     .then_signal_fence_and_flush().unwrap();
        // future.wait(None).unwrap();
        // {
        //     let (tx, rx) = channel();
        //     self.output_buffer.slice(..).map_async(wgpu::MapMode::Read, move |result| {
        //         tx.send(result).unwrap()
        //     });
            self.render_device.poll(wgpu::Maintain::Wait);
        //     rx.recv().unwrap().unwrap();
        // }
        gpu_execution_span.exit();

        
    }

    pub fn swap_buffers(&mut self) {
        (self.input_buffer, self.output_buffer) = (self.output_buffer.clone(), self.input_buffer.clone());
        // self.descriptor_set = PersistentDescriptorSet::new(
        //     &self.descriptor_set_allocator,
        //     self.layout.clone(),
        //     [WriteDescriptorSet::buffer(0, self.input_buffer.clone()), WriteDescriptorSet::buffer(1, self.output_buffer.clone())],
        // ).unwrap();
        self.bind_group = self.render_device.wgpu_device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.input_buffer.as_entire_binding()
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.output_buffer.as_entire_binding()
                },
            ]
        });
    }

    pub fn copy_to_buffer(&self, input: &[Cell]) {
        // let mut content = self.input_buffer.write().unwrap();
        {
            let mut content = self.staging_input_buffer.slice(..).get_mapped_range_mut();
            for (src, dst) in bytemuck::cast_slice(input).iter().zip(content.iter_mut()) {
                *dst = *src;
            }
        }
        let mut encoder = self.render_device.create_command_encoder(&Default::default());
        encoder.copy_buffer_to_buffer(&self.staging_input_buffer, 0, &self.input_buffer, 0, self.staging_input_buffer.size());
        self.staging_input_buffer.unmap();
        self.render_queue.submit([encoder.finish()]);
        self.render_device.poll(wgpu::Maintain::Wait);
    }

    pub fn read_back<S: SliceArg<Dim<[usize; 2]>>>(&self, slice_arg: S) -> ArrayBase<OwnedRepr<Cell>, S::OutDim> {
        let _readback_span = info_span!("readback").entered();
        let mut encoder = self.render_device.create_command_encoder(&Default::default());
        encoder.copy_buffer_to_buffer(&self.output_buffer, 0, &self.staging_output_buffer, 0, self.output_buffer.size());
        self.render_queue.submit([encoder.finish()]);
        self.render_device.poll(wgpu::Maintain::Wait);

        let chunk_from_gpu = {
            let (tx, rx) = channel();
            self.staging_output_buffer.slice(..).map_async(wgpu::MapMode::Read, move |result| {
                tx.send(result).unwrap()
            });
            self.render_device.poll(wgpu::Maintain::Wait);
            rx.recv().unwrap().unwrap();

            // let read_lock = self.output_buffer.read().unwrap();
            let output_data = self.staging_output_buffer.slice(..).get_mapped_range();

            let gpu_chunk_lock = info_span!("gpu_chunk").entered();
            let bytes_from_gpu = ArrayView::from_shape((WORLD_WIDTH, WORLD_WIDTH, mem::size_of::<Cell>()), &output_data).unwrap();
            let cells_from_gpu = bytes_from_gpu.map_axis(ndarray::Axis(2), |bytes| *bytemuck::from_bytes::<Cell>(bytes.as_slice().unwrap()));
            let chunk_from_gpu = cells_from_gpu.slice(slice_arg);
            gpu_chunk_lock.exit();

            chunk_from_gpu.into_owned()
        };

        self.staging_output_buffer.unmap();

        chunk_from_gpu
    }

    pub fn new(buffer_length: usize) -> BiosimComputeShader {
        let instance = wgpu::Instance::new(InstanceDescriptor::default());
        let adapter = instance.request_adapter(&Default::default()).block_on().unwrap();
        let (device, queue) = adapter.request_device(&DeviceDescriptor { required_features: Features::SPIRV_SHADER_PASSTHROUGH, required_limits: Limits { max_storage_buffer_binding_size: 268435456, ..Default::default() }, ..Default::default() }, None).block_on().unwrap();

        let render_device = RenderDevice::from(device);
        let render_queue = RenderQueue(queue.into());
       println!("Selected adapter: {}", adapter.get_info().name);
        // let library = VulkanLibrary::new().unwrap();
        // let instance = Instance::new(library, InstanceCreateInfo {
        //     ..Default::default()
        // }).unwrap();

        // let physical_devices = instance.enumerate_physical_devices().unwrap();

        // let (physical_device, queue_family_index) = physical_devices
        //     .filter_map(|pdev| {
        //         pdev.queue_family_properties()
        //             .iter()
        //             .enumerate()
        //             .find(|(_, q)| q.queue_flags.intersects(&QueueFlags { compute: true, ..Default::default() }))
        //             .map(|(index, _)| (pdev.clone(), index as u32))
        //     })
        //     .next()
        //     .expect("No device with compute capability found");

        // println!("Selected device: {}", physical_device.properties().device_name);
        
        // let supported_features = physical_device.supported_features();
        // if !supported_features.vulkan_memory_model {
        //         panic!("Selected physical device does not support vulkan_memory_model feature required by the shader");
        // }

        // let features = Features {
        //     vulkan_memory_model: true,
        //     ..Features::default()
        // };

        // let device_extensions = DeviceExtensions {
        //     khr_vulkan_memory_model: true,
        //     ..Default::default()
        // };

        // let (device, mut queues) = Device::new(
        //         physical_device,
        //         DeviceCreateInfo {
        //             enabled_features: features,
        //             enabled_extensions: device_extensions,
        //             queue_create_infos: vec![QueueCreateInfo {
        //                 queue_family_index,
        //                 ..Default::default()
        //             }],
        //             ..Default::default()
        //         }
        //     )
        //     .unwrap();

        // let queue = queues.next().unwrap();

        // let spirv_bytes = std::fs::read(env!("biosim_rust_shader.spv")).unwrap();
        // // For some reason, when I just use from_bytes here, we end up with the equivalent of the
        // // `descriptor_requirements` being `[]`, which causes a segfault a bit later on. I'm not
        // // sure at the moment if this is an issue with vulkano or rust-gpu, or something I'm doing
        // // wrong. However, by mimicking the structure of the `EntryPointInfo` generated when we use
        // // `vulkano_shaders:shader!` with a `src` property, we can get the compute shader to load
        // // correctly.
        // let shader = unsafe { ShaderModule::from_bytes_with_data(
        //     device.clone(),
        //     &spirv_bytes, 
        //     Version::major_minor(1, 3), 
        //     [&Capability::Shader, &Capability::VulkanMemoryModel], 
        //     [], 
        //     [(
        //         "main".to_owned(),
        //         ExecutionModel::GLCompute,
        //         EntryPointInfo {
        //             execution: ShaderExecution::Compute, 
        //             descriptor_requirements: [
        //                 (
        //                     (0u32, 0u32), 
        //                     DescriptorRequirements {
        //                         descriptor_types: vec![DescriptorType:: StorageBuffer, DescriptorType :: StorageBufferDynamic],
        //                         descriptor_count: Some(1u32),
        //                         stages: ShaderStages { compute: true, ..Default::default() },
        //                         storage_write: [0u32].into_iter().collect(),
        //                         ..Default::default()
        //                     }
        //                 ),
        //                 (
        //                     (0u32, 1u32), 
        //                     DescriptorRequirements {
        //                         descriptor_types: vec![DescriptorType:: StorageBuffer, DescriptorType :: StorageBufferDynamic],
        //                         descriptor_count: Some(1u32),
        //                         stages: ShaderStages { compute: true, ..Default::default() },
        //                         storage_write: [1u32].into_iter().collect(),
        //                         ..Default::default()
        //                     }
        //                 ),
        //             ].into_iter().collect(),
        //             push_constant_requirements: None,
        //             specialization_constant_requirements: [].into_iter().collect(),
        //             input_interface: ShaderInterface::new_unchecked(vec! []),
        //             output_interface: ShaderInterface::new_unchecked(vec! []),
        //         }
        //     )])}.unwrap();
        let shader = unsafe { render_device.wgpu_device().create_shader_module_spirv(&wgpu::include_spirv_raw!(env!("biosim_rust_shader.spv"))) };

        // let memory_allocator = StandardMemoryAllocator::new_default(device.clone());
        // let input_buffer = CpuAccessibleBuffer::from_iter(
        //     &memory_allocator, 
        //     BufferUsage { storage_buffer: true, ..Default::default() }, 
        //     false, 
        //     vec![Cell::Dead; buffer_length],
        // ).unwrap();
        // let output_buffer = CpuAccessibleBuffer::from_iter(
        //     &memory_allocator, 
        //     BufferUsage { storage_buffer: true, ..Default::default() }, 
        //     false, 
        //     vec![Cell::Dead; buffer_length]
        // ).unwrap();
        let staging_input_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("staging in"),
            // contents: bytemuck::cast_slice(vec![Cell::Dead; buffer_length].as_slice()),
            size: (buffer_length * mem::size_of::<Cell>()) as u64,
            usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: true,
        });
        let input_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("buffer a"),
            size: staging_input_buffer.size(),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let output_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("buffer b"),
            size: input_buffer.size(),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let staging_output_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("staging out"),
            size: output_buffer.size(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let bind_group_layout = render_device.create_bind_group_layout(Some("bind group layout"), &[
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: Some(NonZero::new(input_buffer.size()).unwrap()) },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 3,
                visibility: ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: Some(NonZero::new(output_buffer.size()).unwrap()) },
                count: None,
            },
        ]);

        let pipeline_layout = render_device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("pipeline layout"),
            bind_group_layouts: &[
                &bind_group_layout
            ],
            push_constant_ranges: &[]
        });

        // let pipeline = ComputePipeline::new(device.clone(), shader.entry_point("main").unwrap(), &(), None, |_| {}).unwrap();
        let pipeline = render_device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Main compute pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
        });

        // let layout = pipeline.layout().set_layouts().get(0).unwrap();
        let bind_group = render_device.wgpu_device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: input_buffer.as_entire_binding()
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: output_buffer.as_entire_binding()
                },
            ]
        });
        // let descriptor_set_allocator = StandardDescriptorSetAllocator::new(device.clone());
        // let descriptor_set = PersistentDescriptorSet::new(
        //     &descriptor_set_allocator,
        //     layout.clone(),
        //     [WriteDescriptorSet::buffer(0, input_buffer.clone()), WriteDescriptorSet::buffer(1, output_buffer.clone())],
        // ).unwrap();

        // let buffer_allocator = StandardCommandBufferAllocator::new(device.clone(), Default::default());
        BiosimComputeShader { render_device, render_queue, pipeline, bind_group, staging_input_buffer, input_buffer, output_buffer, staging_output_buffer } // TODO: staging input buffer shouldn't be passed on
    }
}