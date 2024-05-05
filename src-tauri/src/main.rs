// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use arr_macro::arr;
use bevy::{
  prelude::*,
  sprite::Mesh2dHandle,
};
use bevy::sprite::MaterialMesh2dBundle;

use bevy_pancam::{PanCam, PanCamPlugin};
use hex_grid::build_hex_grid_mesh;
use world::{Cell, World};

mod world;
mod hex_grid;

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
