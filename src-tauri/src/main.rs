// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::f32::consts::PI;
use arr_macro::arr;
use bevy::{
  prelude::*,
  render::{
      mesh::Indices,
      render_asset::RenderAssetUsages,
      render_resource::PrimitiveTopology,
  },
  sprite::{
      MaterialMesh2dBundle, Mesh2dHandle
  },
};

use bevy_pancam::{PanCam, PanCamPlugin};
use world::{Cell, World, WORLD_WIDTH};

mod world;

fn main() {
  App::new()
    .add_plugins((DefaultPlugins, HelloPlugin))
    .run();
}

pub struct HelloPlugin;

impl Plugin for HelloPlugin {
  fn build(&self, app: &mut App) {
    app.add_plugins(PanCamPlugin::default())
      .insert_resource(WorldTickTimer(Timer::from_seconds(0.5, TimerMode::Repeating)))
      .add_systems(Startup, setup)
      .add_systems(Update, update_world);
  }
}

#[derive(Resource)]
struct WorldTickTimer(Timer);

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
  commands.spawn(Camera2dBundle::default())
    .insert(PanCam::default());

  commands.spawn(MaterialMesh2dBundle {
    mesh: meshes.add(build_hex_grid_mesh()).into(),
    material: materials.add(Color::WHITE),
    ..default()
  }).insert(WorldComponent(World::new_random()));
} 

#[derive(Component)]
struct WorldComponent(World);

const NUM_VERTS_PER_HEX: u32 = 6;
fn build_hex_grid_mesh() -> Mesh {
  let mut mesh = Mesh::new(
    PrimitiveTopology::TriangleList,
    RenderAssetUsages::all()
  );
  mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, get_positions());
  mesh.insert_indices(Indices::U32(get_indices()));
  mesh.insert_attribute(
    Mesh::ATTRIBUTE_COLOR, 
    vec![
      Color::WHITE.as_linear_rgba_f32();
      WORLD_WIDTH * WORLD_WIDTH * NUM_VERTS_PER_HEX as usize
    ]
  );

  mesh
}

fn get_positions() -> Vec<[f32; 3]> {
  let mut positions: Vec<[f32; 3]> = vec![];

  let delta_y = (3.0f32).sqrt() / 2.;
  for x in 0..WORLD_WIDTH {
    for y in 0..WORLD_WIDTH {
      let x_center = (3. * x as f32) + (3. * (y as f32 / 2.));
      let y_center = delta_y * y as f32;

      for v in 0..6 {
        positions.push(get_hex_vertex_pos(x_center, y_center, v))
      }
    }
  }

  positions
}

fn get_hex_vertex_pos(x_center: f32, y_center: f32, vertex_index: usize) -> [f32; 3] {
  const ANGLE_MULTIPLIER: f32 = PI / 3.;

  let angle = ANGLE_MULTIPLIER * vertex_index as f32;
  [angle.cos() + x_center, angle.sin() + y_center, 0.]
}

fn get_indices() -> Vec<u32> {
  let mut indices: Vec<u32> = vec![];

  for x in 0..WORLD_WIDTH {
    for y in 0..WORLD_WIDTH {
      let offset = (x as u32 + (y * WORLD_WIDTH) as u32) * NUM_VERTS_PER_HEX;
      indices.extend_from_slice(&[
        offset + 0, offset + 1, offset + 5,
        offset + 1, offset + 2, offset + 5,
        offset + 2, offset + 4, offset + 5,
        offset + 2, offset + 3, offset + 4,
      ]);
    }
  }

  indices
}

fn update_world(mut meshes: ResMut<Assets<Mesh>>, mut timer: ResMut<WorldTickTimer>, time: Res<Time>, mut query: Query<(&mut WorldComponent, &mut Mesh2dHandle)>) {
  if !timer.0.tick(time.delta()).just_finished() {
    return;
  }

  for (mut world_component, mesh_handle) in &mut query {
    let Some(mesh) = meshes.get_mut(mesh_handle.0.id()) else {
      break;
    };

    let cells: Vec<&Cell> = world_component.0.cells.iter().flat_map(|row| row.iter()).collect();
    let colors: Vec<[f32; 4]> = cells.iter().flat_map(|cell| 
      if **cell == Cell::Alive { arr![[0., 0., 0., 1.]; 6] } else { arr![[1., 1., 1., 1.]; 6] }
    ).collect();

    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);

    world_component.0 = world_component.0.tick();
  }
}
