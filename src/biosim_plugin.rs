use arr_macro::arr;
use bevy::{app::{App, Plugin, Startup, Update}, asset::Assets, core_pipeline::core_2d::Camera2dBundle, ecs::{component::Component, system::{Commands, Query, Res, ResMut, Resource}}, render::{color::Color, mesh::Mesh}, sprite::{ColorMaterial, MaterialMesh2dBundle, Mesh2dHandle}, time::{Time, Timer, TimerMode}};
use bevy_pancam::{PanCam, PanCamPlugin};

use crate::{hex_grid::build_hex_grid_mesh, world::{Cell, World}};
use bevy::prelude::*;


pub struct BiosimPlugin;

impl Plugin for BiosimPlugin {
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