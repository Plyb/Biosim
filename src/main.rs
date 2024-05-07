// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use bevy::prelude::*;

use biosim_plugin::BiosimPlugin;

mod world;
mod hex_grid;
mod biosim_plugin;

fn main() {
  App::new()
    .add_plugins((DefaultPlugins, BiosimPlugin))
    .run();
}
