use std::vec;

use bevy::{app::{App, Plugin, Startup, Update}, asset::Assets, core_pipeline::core_2d::Camera2dBundle, ecs::{component::Component, system::{Commands, Query, Res, ResMut, Resource}}, render::{mesh::Mesh, render_asset::RenderAssetUsages, render_resource::{AsBindGroup, Buffer, Extent3d, ShaderRef, TextureDimension, TextureFormat}}, sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle}, time::{Time, Timer, TimerMode}};
use bevy_pancam::{PanCam, PanCamPlugin};
use biosim_core::{hex_grid::{uv_to_hexel_coord, uv_to_rect_grid_coord, world_space_to_uv}, world::{Cell, WorldCoord, WorldOffset}, WORLD_WIDTH, WORLD_WIDTH_MULTIPLER};
use ndarray::{s, ArrayView, ArrayViewMut};

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
    let world_material = WorldMaterial { hexels: image_handle }; // TODO: add buffer

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
    #[texture(0)]
    #[sampler(1)]
    hexels: Handle<Image>,

    // #[storage(3, read_only, buffer)]
    // buffer: Buffer,
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
    mut world_query: Query<(&mut WorldComponent, &Handle<WorldMaterial>)>,
    mut compute_shader: ResMut<BiosimComputeShader>,
    camera_query: Query<&GlobalTransform, With<Camera>>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }
    let _true_update_world_span = info_span!("update_world_past_timer").entered();

    let camera_pos = camera_query.get_single().unwrap().translation();

    for (mut world_component, mesh_handle) in &mut world_query {
        let Some(world_material) = materials.get_mut(mesh_handle.id()) else {
            break;
        };

        let Some(image) = images.remove(world_material.hexels.id()) else {
            break;
        };

        let tick_span = info_span!("ticking").entered();
        let (u, v) = world_space_to_uv(camera_pos.x, camera_pos.y);
        let center = if cfg!(feature = "rect_grid") {
            uv_to_rect_grid_coord(u, v)
        } else {
            uv_to_hexel_coord(u, v)
        };

        const CHUNK_RADIUS: i32 = 256;
        let low = if cfg!(feature = "cpu") {
            WorldCoord::min()
        } else {
            center.add_clamped(WorldOffset { x: -CHUNK_RADIUS, y: -CHUNK_RADIUS })
        };
        let high = if cfg!(feature = "cpu") {
            WorldCoord::max()
        } else {
            center.add_clamped(WorldOffset { x: CHUNK_RADIUS, y: CHUNK_RADIUS })
        };

        let cell_chunk = if cfg!(feature = "cpu") {
            world_component.0 = tick(&world_component.0);
            ArrayView::from_shape((high.y - low.y, high.x - low.x), world_component.0.as_slice()).unwrap().to_owned()
        } else {
            compute_shader.dispatch();
            let slice_arg = s![low.y..high.y, low.x..high.x];
            let cells_from_gpu = compute_shader.read_back(slice_arg);
            compute_shader.swap_buffers();
            cells_from_gpu
        };

        let new_bytes_flat = cell_chunk.iter().flat_map(|cell| if *cell == Cell::Alive { [0, 0, 0, 255] } else { [255, 255, 255, 255] }).collect::<Vec<u8>>();
        let new_bytes = ArrayView::from_shape((high.y - low.y, high.x - low.x, 4), &new_bytes_flat.as_slice()).unwrap();
        
        let image_lock = info_span!("image").entered();
        let mut dyn_img = Image::try_into_dynamic(image).unwrap();
        let mut image_bytes = ArrayViewMut::from_shape((WORLD_WIDTH, WORLD_WIDTH, 4), dyn_img.as_mut_rgba8().unwrap()).unwrap();
        image_lock.exit();

        let assign_lock = info_span!("assign").entered();
        image_bytes.slice_mut(s![low.y..high.y, low.x..high.x, ..]).assign(&new_bytes);
        assign_lock.exit();


        let updated_image = Image::from_dynamic(
            dyn_img,
            true,
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD
        );
        world_material.hexels = images.add(updated_image);

        tick_span.exit();
  }
}
