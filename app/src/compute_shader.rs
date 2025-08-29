use std::{mem, num::NonZero, sync::mpsc::channel};

use bevy::{ecs::system::Resource, log::info_span, render::{render_resource::{Buffer, ComputePipeline}, renderer::{RenderDevice, RenderQueue}}};
use biosim_core::{world::Cell, WORLD_WIDTH};
use ndarray::{ArrayBase, ArrayView, Dim, OwnedRepr, SliceArg};
use wgpu::{BindGroup, BindGroupLayoutEntry, BufferDescriptor, PipelineLayoutDescriptor, ShaderStages};

#[derive(Resource)]
pub struct BiosimComputeShader {
    render_device: RenderDevice,
    render_queue: RenderQueue,
    pipeline: ComputePipeline,
    bind_group: BindGroup,
    staging_input_buffer: Buffer,
    input_buffer: Buffer,
    output_buffer: Buffer,
    staging_output_buffer: Buffer,
}

impl BiosimComputeShader {
    pub fn dispatch(&self) {
        const THREADS_PER_WORKGROUP: u32 = 32;

        let mut encoder = self.render_device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.dispatch_workgroups(WORLD_WIDTH as u32 / THREADS_PER_WORKGROUP, WORLD_WIDTH as u32 / THREADS_PER_WORKGROUP, 1);
        }

        self.render_queue.submit([encoder.finish()]);
        
        let gpu_execution_span = info_span!("gpu").entered();
        self.render_device.poll(wgpu::Maintain::Wait);
        gpu_execution_span.exit();
    }

    pub fn get_cells_buffer(&self) -> Buffer {
        self.input_buffer.clone()
    }

    pub fn swap_buffers(&mut self) {
        (self.input_buffer, self.output_buffer) = (self.output_buffer.clone(), self.input_buffer.clone());
        self.bind_group = Self::create_bind_group(&self.render_device, &self.pipeline, &self.input_buffer, &self.output_buffer);
    }

    fn create_bind_group(render_device: &RenderDevice, pipeline: &ComputePipeline, input_buffer: &Buffer, output_buffer: &Buffer) -> BindGroup {
        render_device.wgpu_device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: input_buffer.as_entire_binding()
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: output_buffer.as_entire_binding()
                },
            ]
        })
    }

    pub fn copy_to_buffer(&self, input: &[Cell]) {
        {
            self.map_buffer(&self.staging_input_buffer, wgpu::MapMode::Write);

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

    #[allow(dead_code)]
    pub fn read_back<S: SliceArg<Dim<[usize; 2]>>>(&self, slice_arg: S) -> ArrayBase<OwnedRepr<Cell>, S::OutDim> {
        let _readback_span = info_span!("readback").entered();
        let mut encoder = self.render_device.create_command_encoder(&Default::default());
        encoder.copy_buffer_to_buffer(&self.output_buffer, 0, &self.staging_output_buffer, 0, self.output_buffer.size());
        self.render_queue.submit([encoder.finish()]);
        self.render_device.poll(wgpu::Maintain::Wait);

        let chunk_from_gpu = {
            self.map_buffer(&self.staging_output_buffer, wgpu::MapMode::Read);

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

    fn map_buffer(&self, buffer: &Buffer, mode: wgpu::MapMode) {
        let (tx, rx) = channel();
        buffer.slice(..).map_async(mode, move |result| {
            tx.send(result).unwrap()
        });
        self.render_device.poll(wgpu::Maintain::Wait);
        rx.recv().unwrap().unwrap();
    }

    pub fn new(buffer_length: usize, render_device: RenderDevice, render_queue: RenderQueue) -> BiosimComputeShader {
        let shader = unsafe { render_device.wgpu_device().create_shader_module_spirv(&wgpu::include_spirv_raw!(env!("biosim_rust_shader.spv"))) };

        // wgpu wants us to use staging buffers to transfer data between the cpu and gpu. Beyond our staging
        // buffers, we have two buffers to form a swappable buffer pair, where input is treated as readonly
        // and output as writeonly. The two buffers are swapped each step of the simulation.
        let staging_input_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("staging in"),
            size: (buffer_length * mem::size_of::<Cell>()) as u64,
            usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let input_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("buffer a"),
            size: staging_input_buffer.size(),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
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

        // It seems like rust-gpu doesn't quite mark the SPIR-V it generates correctly or something, because wgpu can't
        // infer the layout correctly, which is why we are manually building it and passing it here.
        let bind_group_layout = render_device.create_bind_group_layout(Some("bind group layout"), &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE | ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: Some(NonZero::new(input_buffer.size()).unwrap()) },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::COMPUTE | ShaderStages::FRAGMENT,
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

        let pipeline = render_device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Main compute pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
        });
        let bind_group = Self::create_bind_group(&render_device, &pipeline, &input_buffer, &output_buffer);

        BiosimComputeShader { render_device, render_queue, pipeline, bind_group, staging_input_buffer, input_buffer, output_buffer, staging_output_buffer }
    }
}