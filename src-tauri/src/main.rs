// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::{thread, time};

use tauri::{Manager, Window};
use world::World;

mod world;

fn main() {
  tauri::Builder::default()
    .setup(|app| {
      let main_window = app.get_window("main").unwrap();

      thread::spawn(move || {
        start_simulation(main_window)
      });

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

fn start_simulation(window : Window) {
  let mut world = World::new_random();

  loop {
    window.emit("update-world", &world).unwrap();
    world = world.tick();
    thread::sleep(time::Duration::from_millis(500));
  }
}
