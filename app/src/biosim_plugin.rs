use std::vec;

use bevy::{app::{App, Plugin, Startup, Update}, asset::Assets, core_pipeline::core_2d::Camera2dBundle, ecs::{component::Component, system::{Commands, Query, Res, ResMut, Resource}}, render::{mesh::Mesh, render_asset::RenderAssetUsages, render_resource::{AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat}}, sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle}, time::{Time, Timer, TimerMode}};
use bevy_pancam::{PanCam, PanCamPlugin};
use biosim_core::{world::Cell, WORLD_WIDTH};

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

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<WorldMaterial>>, mut images: ResMut<Assets<Image>>) {
    commands.spawn(Camera2dBundle::default())
        .insert(PanCam::default());

    let cells = new_random();
    let world_component = WorldComponent(cells);

    let compute_shader = BiosimComputeShader::new(WORLD_WIDTH * WORLD_WIDTH);
    compute_shader.copy_to_buffer(&world_component.0);
    commands.insert_resource(compute_shader);

    let image = Image::new(
        Extent3d { width: WORLD_WIDTH as u32, height: WORLD_WIDTH as u32, depth_or_array_layers: 1 },
        TextureDimension::D2,
        vec![u8::MAX; WORLD_WIDTH * WORLD_WIDTH * 4],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD
    );
    let image_handle = images.add(image); println!("Original: {:?}", image_handle.id());
    let world_material = WorldMaterial { hexels: image_handle };

    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(Rectangle::from_size(Vec2 { x: WORLD_WIDTH as f32 * 6.0, y: WORLD_WIDTH as f32 })).into(),
        material: materials.add(world_material),
        ..default()
    }).insert(world_component);
} 

#[derive(Component)]
struct WorldComponent(Vec<Cell>);

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

fn update_world(
    mut materials: ResMut<Assets<WorldMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut timer: ResMut<WorldTickTimer>,
    time: Res<Time>,
    world_query: Query<(&WorldComponent,&Handle<WorldMaterial>)>,
    mut compute_shader: ResMut<BiosimComputeShader>,
    camera_query: Query<&GlobalTransform, With<Camera>>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }
    let _true_update_world_span = info_span!("update_world_past_timer").entered();

    let camera_pos = camera_query.get_single().unwrap().translation();

    for (_, mesh_handle) in &world_query {
        let Some(world_material) = materials.get_mut(mesh_handle.id()) else {
            break;
        };

        let Some(image) = images.remove(world_material.hexels.id()) else {
            break;
        };

        let tick_span = info_span!("ticking").entered();
        compute_shader.dispatch();
        let updated_image = compute_shader.read_back_to_image(camera_pos, image);
        world_material.hexels = images.add(updated_image);
        compute_shader.swap_buffers();
        // world_component.0 = tick(&world_component.0);
        tick_span.exit();
  }
}
