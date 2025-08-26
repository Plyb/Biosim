use std::sync::Arc;

use bevy::{app::{App, Plugin, Startup, Update}, asset::Assets, core_pipeline::core_2d::Camera2dBundle, ecs::{component::Component, system::{Commands, Query, Res, ResMut, Resource}}, render::{mesh::Mesh, render_asset::RenderAssetUsages, render_resource::{AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat}}, sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle}, time::{Time, Timer, TimerMode}};
use bevy_pancam::{PanCam, PanCamPlugin};
use biosim_core::{world::Cell, WORLD_WIDTH};
use vulkano::{buffer::{BufferUsage, CpuAccessibleBuffer}, command_buffer::{allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage}, descriptor_set::{allocator::StandardDescriptorSetAllocator, layout::DescriptorType, PersistentDescriptorSet, WriteDescriptorSet}, device::{Device, DeviceCreateInfo, DeviceExtensions, Features, Queue, QueueCreateInfo, QueueFlags}, instance::{Instance, InstanceCreateInfo}, memory::allocator::StandardMemoryAllocator, pipeline::{ComputePipeline, Pipeline, PipelineBindPoint}, shader::{spirv::{Capability, ExecutionModel}, DescriptorRequirements, EntryPointInfo, ShaderExecution, ShaderInterface, ShaderModule, ShaderStages}, sync::{self, GpuFuture}, Version, VulkanLibrary};

use crate::world::World;
use bevy::prelude::*;


pub struct BiosimPlugin;

impl Plugin for BiosimPlugin {
  fn build(&self, app: &mut App) {
    app.add_plugins((PanCamPlugin::default(), Material2dPlugin::<WorldMaterial>::default()))
      .insert_resource(WorldTickTimer(Timer::from_seconds(0.5, TimerMode::Repeating)))
      .add_systems(Startup, setup)
      .add_systems(Update, update_world);
  }
}

#[derive(Resource)]
struct WorldTickTimer(Timer);

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<WorldMaterial>>) {
  commands.spawn(Camera2dBundle::default())
    .insert(PanCam::default());

  commands.spawn(MaterialMesh2dBundle {
    mesh: meshes.add(Rectangle::from_size(Vec2 { x: WORLD_WIDTH as f32 * 6.0, y: WORLD_WIDTH as f32 })).into(),
    material: materials.add(WorldMaterial { hexels: default() }),
    ..default()
  }).insert(WorldComponent(World::new_random()));

  let mut compute_shader = BiosimComputeShader::new(4);
  compute_shader.dispatch(&[1.0, 2.0, 3.0, 4.0]);
  commands.insert_resource(compute_shader);
} 

#[derive(Component)]
struct WorldComponent(World);

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct WorldMaterial {
  #[texture(0)]
  #[sampler(1)]
  hexels: Handle<Image>
}

impl Material2d for WorldMaterial {
  fn fragment_shader() -> ShaderRef {
    env!("biosim_rust_shader.spv").into()
  }
}

fn update_world(mut materials: ResMut<Assets<WorldMaterial>>, mut images: ResMut<Assets<Image>>, mut timer: ResMut<WorldTickTimer>, time: Res<Time>, mut query: Query<(&mut WorldComponent, &mut Handle<WorldMaterial>)>, mut compute_shader: ResMut<BiosimComputeShader>) {
  if !timer.0.tick(time.delta()).just_finished() {
    return;
  }
  let _true_update_world_span = info_span!("update_world_past_timer").entered();

  for (mut world_component, mesh_handle) in &mut query {
    let Some(world_material) = materials.get_mut(mesh_handle.id()) else {
      break;
    };

    let collection_span = info_span!("collection").entered();
    let cells: Vec<&Cell> = world_component.0.cells.iter().flat_map(|row| row.iter()).collect();
    let colors: Vec<u8> = cells.iter().flat_map(|cell| 
      if **cell == Cell::Alive { [0, 0, 0, 255] } else { [255, 255, 255, 255] }
    ).collect();
    collection_span.exit();

    let image = Image::new(
      Extent3d { width: WORLD_WIDTH as u32, height: WORLD_WIDTH as u32, depth_or_array_layers: 1 },
      TextureDimension::D2,
      colors,
      TextureFormat::Rgba8Unorm,
      RenderAssetUsages::RENDER_WORLD
    );
    world_material.hexels = images.add(image);

    let tick_span = info_span!("ticking").entered();
    world_component.0 = world_component.0.tick();
    tick_span.exit();

    let last_output = compute_shader.last_output.clone();
    println!("Result: {:?}", compute_shader.dispatch(&last_output));
  }
}

struct BiosimComputeShader {
  device: Arc<Device>,
  queue: Arc<Queue>,
  pipeline: Arc<ComputePipeline>,
  descriptor_set: Arc<PersistentDescriptorSet>,
  buffer_allocator: StandardCommandBufferAllocator,
  buffer: Arc<CpuAccessibleBuffer<[f32]>>, // Note that this means we have a fixed sized for our buffer. If we want variable size, we'd need to rebuild the buffer each time.
  last_output: Vec<f32>
}

impl BiosimComputeShader {
  fn dispatch(&mut self, input: &[f32]) -> Vec<f32> {
    let length = input.len() as u32;

    self.copy_to_buffer(input);

    let mut builder = AutoCommandBufferBuilder::primary(
        &self.buffer_allocator,
        self.queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
      ).unwrap();
    builder.bind_pipeline_compute(self.pipeline.clone())
      .bind_descriptor_sets(PipelineBindPoint::Compute, self.pipeline.layout().clone(), 0, self.descriptor_set.clone())
      .dispatch([length, 1, 1])
      .unwrap();

    let command_buffer = builder.build().unwrap();

    let future = sync::now(self.device.clone())
      .then_execute(self.queue.clone(), command_buffer).unwrap()
      .then_signal_fence_and_flush().unwrap();

    future.wait(None).unwrap();

    let content = self.buffer.read().unwrap().to_vec();
    self.last_output = content.clone();
    content
  }

  fn copy_to_buffer(&self, input: &[f32]) {
    let mut content = self.buffer.write().unwrap();
    for (src, dst) in input.iter().zip(content.iter_mut()) {
      *dst = *src;
    }
  }

  fn new(buffer_length: usize) -> BiosimComputeShader {
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
          descriptor_requirements: [(
            (0u32, 0u32), 
            DescriptorRequirements {
              descriptor_types: vec![DescriptorType:: StorageBuffer, DescriptorType :: StorageBufferDynamic],
              descriptor_count: Some(1u32),
              stages: ShaderStages { compute: true, ..Default::default() },
              storage_write: [0u32].into_iter().collect(),
              ..Default::default()
            }
          )].into_iter().collect(),
          push_constant_requirements: None,
          specialization_constant_requirements: [].into_iter().collect(),
          input_interface: ShaderInterface::new_unchecked(vec! []),
          output_interface: ShaderInterface::new_unchecked(vec! []),
        }
      )])}.unwrap();

    let data = vec![0.0f32; buffer_length];
    let data_iter = data.clone().into_iter();

    let memory_allocator = StandardMemoryAllocator::new_default(device.clone());
    let buffer = CpuAccessibleBuffer::from_iter(
      &memory_allocator, 
      BufferUsage { storage_buffer: true, ..Default::default() }, 
      false, 
      data_iter
    ).unwrap();

    let pipeline = ComputePipeline::new(device.clone(), shader.entry_point("main").unwrap(), &(), None, |_| {}).unwrap();
    let layout = pipeline.layout().set_layouts().get(0).unwrap();
    let descriptor_set_allocator = StandardDescriptorSetAllocator::new(device.clone());
    let descriptor_set = PersistentDescriptorSet::new(
      &descriptor_set_allocator,
      layout.clone(),
      [WriteDescriptorSet::buffer(0, buffer.clone())],
    ).unwrap();

    let buffer_allocator = StandardCommandBufferAllocator::new(device.clone(), Default::default());
    BiosimComputeShader { device, queue, pipeline, descriptor_set, buffer_allocator, buffer, last_output: data }
  }
}

impl Resource for BiosimComputeShader {}
