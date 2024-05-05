// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::{f32::consts::PI, thread, time};
use arr_macro::arr;
use bevy::{
  core_pipeline::core_2d::Transparent2d,
  prelude::*,
  render::{
      mesh::{Indices, MeshVertexAttribute},
      render_asset::{RenderAssetUsages, RenderAssets},
      render_phase::{AddRenderCommand, DrawFunctions, RenderPhase, SetItemPipeline},
      render_resource::{
          BlendState, ColorTargetState, ColorWrites, Face, FragmentState, FrontFace,
          MultisampleState, PipelineCache, PolygonMode, PrimitiveState, PrimitiveTopology,
          PushConstantRange, RenderPipelineDescriptor, ShaderStages, SpecializedRenderPipeline,
          SpecializedRenderPipelines, TextureFormat, VertexBufferLayout, VertexFormat,
          VertexState, VertexStepMode,
      },
      texture::BevyDefault,
      view::{ExtractedView, ViewTarget, VisibleEntities},
      Extract, Render, RenderApp, RenderSet,
  },
  sprite::{
      extract_mesh2d, DrawMesh2d, Material2dBindGroupId, MaterialMesh2dBundle, Mesh2d, Mesh2dHandle, Mesh2dPipeline, Mesh2dPipelineKey, Mesh2dRenderPlugin, Mesh2dTransforms, MeshFlags, RenderMesh2dInstance, RenderMesh2dInstances, SetMesh2dBindGroup, SetMesh2dViewBindGroup
  },
  utils::FloatOrd,
};

use bevy_pancam::{PanCam, PanCamPlugin};
use tauri::{LogicalSize, Manager, Size, Window};
use world::{Cell, World, WORLD_WIDTH};

mod world;

fn main() {
  App::new()
    .add_plugins((DefaultPlugins, HelloPlugin))
    .run();
  // tauri::Builder::default()
  //   .setup(|app| {
  //     let main_window = app.get_window("main").unwrap();
  //     let _ = main_window.set_size(
  //       Size::Logical(LogicalSize { width: 500.0, height: 500.0})
  //     );

  //     thread::spawn(move || {
  //       start_simulation(main_window)
  //     });

  //     Ok(())
  //   })
  //   .invoke_handler(tauri::generate_handler![get_world_width])
  //   .run(tauri::generate_context!())
  //   .expect("error while running tauri application");
}

pub struct HelloPlugin;

impl Plugin for HelloPlugin {
  fn build(&self, app: &mut App) {
    app.add_plugins(PanCamPlugin::default())
      .insert_resource(GreetTimer(Timer::from_seconds(0.5, TimerMode::Repeating)))
      .add_systems(Startup, setup)
      .add_systems(Update, update_world);
  }
}

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Name(String);

#[derive(Resource)]
struct GreetTimer(Timer);

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
  commands.spawn(Camera2dBundle::default())
    .insert(PanCam::default());;

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

fn add_people(mut commands: Commands) {
  commands.spawn((Person, Name("Elaina Proctor".to_string())));
  commands.spawn((Person, Name("Renzo Hume".to_string())));
  commands.spawn((Person, Name("Zayna Nieves".to_string())));
}

fn greet_people(time: Res<Time>, mut timer: ResMut<GreetTimer>, query: Query<&Name, With<Person>>) {
  if timer.0.tick(time.delta()).just_finished() {
    for name in &query {
        println!("hello {}!", name.0);
    }
  }
}

fn update_world(mut meshes: ResMut<Assets<Mesh>>, mut timer: ResMut<GreetTimer>, time: Res<Time>, mut query: Query<(&mut WorldComponent, &mut Mesh2dHandle)>) {
  if !timer.0.tick(time.delta()).just_finished() {
    return;
  }

  for (mut world_component, mut mesh_handle) in &mut query {
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

fn update_people(mut query: Query<&mut Name, With<Person>>) {
  for mut name in &mut query {
      if name.0 == "Elaina Proctor" {
          name.0 = "Elaina Hume".to_string();
          break; // We don’t need to change any other names
      }
  }
}

fn start_simulation(window : Window) {
  let mut world = World::new_random();

  loop {
    window.emit("update-world", &world).unwrap();
    world = world.tick();
    thread::sleep(time::Duration::from_millis(500));
  }
}

#[tauri::command]
fn get_world_width() -> usize {
  WORLD_WIDTH
}
