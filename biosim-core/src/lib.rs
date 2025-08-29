#![no_std]

pub mod world;
pub mod util;
pub mod hex_grid;

pub const WORLD_WIDTH: usize = 512;
#[cfg(feature = "rect_grid")]
pub const WORLD_WIDTH_MULTIPLER: f32 = 1.0;
#[cfg(not(feature = "rect_grid"))]
pub const WORLD_WIDTH_MULTIPLER: f32 = 6.0;
