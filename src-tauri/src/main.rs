// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::{f32::consts::PI, thread, time};
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

use tauri::{LogicalSize, Manager, Size, Window};
use world::{World, WORLD_WIDTH};

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
    app.insert_resource(GreetTimer(Timer::from_seconds(2.0, TimerMode::Repeating)))
      .add_systems(Startup, setup);
  }
}

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Name(String);

#[derive(Resource)]
struct GreetTimer(Timer);

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
  commands.spawn(Camera2dBundle::default());

  let mut star = Mesh::new(
    PrimitiveTopology::TriangleList,
    RenderAssetUsages::RENDER_WORLD,
  );

  let mut v_pos = vec![[0.0, 0.0, 0.0]];
    for i in 0..10 {
        // The angle between each vertex is 1/10 of a full rotation.
        let a = i as f32 * PI / 5.0;
        // The radius of inner vertices (even indices) is 100. For outer vertices (odd indices) it's 200.
        let r = (1 - i % 2) as f32 * 100.0 + 100.0;
        // Add the vertex position.
        v_pos.push([r * a.sin(), r * a.cos(), 0.0]);
    }
    // Set the position attribute
    star.insert_attribute(Mesh::ATTRIBUTE_POSITION, v_pos);
    // And a RGB color attribute as well
    let mut v_color: Vec<[f32; 4]> = vec![Color::BLACK.as_linear_rgba_f32()];
    v_color.extend_from_slice(&[Color::YELLOW.as_linear_rgba_f32(); 10]);
    star.insert_attribute(
        Mesh::ATTRIBUTE_COLOR,
        v_color,
    );

    let mut indices = vec![0, 1, 10];
    for i in 2..=10 {
        indices.extend_from_slice(&[0, i, i - 1]);
    }
    star.insert_indices(Indices::U32(indices));

    commands.spawn(MaterialMesh2dBundle {
      mesh: meshes.add(star).into(),
      material: materials.add(Color::WHITE),
      ..default()
    });

  //   commands.spawn((
  //     // We use a marker component to identify the custom colored meshes
  //     Mesh2d,
  //     // The `Handle<Mesh>` needs to be wrapped in a `Mesh2dHandle` to use 2d rendering instead of 3d
  //     Mesh2dHandle(meshes.add(star)),
  //     // This bundle's components are needed for something to be rendered
  //     SpatialBundle::INHERITED_IDENTITY,
  // ));
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
