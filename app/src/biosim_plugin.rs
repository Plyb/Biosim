use std::vec;

use bevy::{app::{App, Plugin, Startup, Update}, asset::Assets, core_pipeline::core_2d::Camera2dBundle, ecs::{component::Component, system::{Commands, Query, Res, ResMut, Resource}}, render::{mesh::Mesh, render_resource::{AsBindGroup, Buffer, ShaderRef}, renderer::{RenderDevice, RenderQueue}}, sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle}, time::{Time, Timer, TimerMode}};
use bevy_pancam::{PanCam, PanCamPlugin};
use biosim_core::{world::Cell, WORLD_WIDTH, WORLD_WIDTH_MULTIPLER};

use crate::world::{new_random, tick};
use crate::compute_shader::BiosimComputeShader;
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

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<WorldMaterial>>, render_device: Res<RenderDevice>, render_queue: Res<RenderQueue>) {
    commands.spawn(Camera2dBundle::default())
        .insert(PanCam::default());

    let cells = new_random();
    let world_component = WorldComponent(cells);

    let compute_shader = BiosimComputeShader::new(WORLD_WIDTH * WORLD_WIDTH, render_device.clone(), render_queue.clone());
    compute_shader.copy_to_buffer(&world_component.0);

    let world_material = WorldMaterial { buffer: compute_shader.get_cells_buffer() };
    commands.insert_resource(compute_shader);

    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(Rectangle::from_size(Vec2 { x: WORLD_WIDTH as f32 * WORLD_WIDTH_MULTIPLER, y: WORLD_WIDTH as f32 })).into(),
        material: materials.add(world_material),
        ..default()
    }).insert(world_component);
} 

#[derive(Component)]
struct WorldComponent(Vec<Cell>);

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct WorldMaterial {
    #[storage(0, read_only, buffer)]
    buffer: Buffer,
}

impl Material2d for WorldMaterial {
    fn fragment_shader() -> ShaderRef {
        env!("biosim_rust_shader.spv").into()
    }
}

fn update_world(
    mut materials: ResMut<Assets<WorldMaterial>>,
    mut timer: ResMut<WorldTickTimer>,
    time: Res<Time>,
    mut world_query: Query<(&mut WorldComponent, &Handle<WorldMaterial>)>,
    mut compute_shader: ResMut<BiosimComputeShader>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }
    let _true_update_world_span = info_span!("update_world_past_timer").entered();

    for (mut world_component, mesh_handle) in &mut world_query {
        let Some(world_material) = materials.get_mut(mesh_handle.id()) else {
            break;
        };

        let tick_span = info_span!("ticking").entered();

        if cfg!(feature = "cpu") {
            world_component.0 = tick(&world_component.0);
            compute_shader.copy_to_buffer(&world_component.0);
        } else {
            compute_shader.dispatch();
            compute_shader.swap_buffers();
            world_material.buffer = compute_shader.get_cells_buffer();
        };

        tick_span.exit();
  }
}
