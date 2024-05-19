use arr_macro::arr;
use bevy::{app::{App, Plugin, Startup, Update}, asset::Assets, core_pipeline::core_2d::Camera2dBundle, ecs::{component::Component, system::{Commands, Query, Res, ResMut, Resource}}, render::{color::Color, mesh::Mesh, render_asset::RenderAssetUsages, render_resource::{AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat}, texture::{self, ImageType}}, sprite::{ColorMaterial, Material2d, Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle}, time::{Time, Timer, TimerMode}};
use bevy_pancam::{PanCam, PanCamPlugin};

use crate::{hex_grid::build_hex_grid_mesh, world::{Cell, World, WORLD_WIDTH}};
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
      "shaders/world.wgsl".into()
  }
}

fn update_world(mut materials: ResMut<Assets<WorldMaterial>>, mut images: ResMut<Assets<Image>>, mut timer: ResMut<WorldTickTimer>, time: Res<Time>, mut query: Query<(&mut WorldComponent, &mut Handle<WorldMaterial>)>) {
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

    // let attr_span = info_span!("inserting attribute").entered();
    // mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    // attr_span.exit();

    let tick_span = info_span!("ticking").entered();
    world_component.0 = world_component.0.tick();
    tick_span.exit();
  }
}