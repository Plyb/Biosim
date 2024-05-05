// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::{thread, time};
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

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
  commands.spawn(MaterialMesh2dBundle {
    mesh: meshes.add(Rectangle::default()).into(),
    transform: Transform::default().with_scale(Vec3::splat(128.)),
    material: materials.add(Color::PURPLE),
    ..default()
  });
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
