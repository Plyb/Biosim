#![allow(unexpected_cfgs)]
#![cfg_attr(target_arch = "spirv", no_std)]

use biosim_core::{world::Cell, WORLD_WIDTH};
use libm::floorf;
use spirv_std::{glam::{vec2, vec4, UVec3, Vec2, Vec3, Vec4, IVec3}, image::Image2d, spirv, Sampler};

#[spirv(fragment)]
pub fn fragment(
  _: Vec3,
  _: Vec3,
  uv: Vec2,
  #[spirv(descriptor_set = 2, binding = 0)] 
  material_color_texture: &Image2d,
  #[spirv(descriptor_set = 2, binding = 1)]
  material_sampler: &Sampler,
  output: &mut Vec4
) {
  hex_grid(uv, material_color_texture, material_sampler, output);
}

// fn rect_grid(uv: Vec2, material_color_texture: &Image2d, material_sampler: &Sampler, output: &mut Vec4) {
//   let world_width = WORLD_WIDTH as f32;
//   *output = material_color_texture.sample(*material_sampler, ((uv * world_width).floor() + 0.5) / world_width)
// }

fn hex_grid(uv: Vec2, material_color_texture: &Image2d, material_sampler: &Sampler, output: &mut Vec4) {
  let world_width = WORLD_WIDTH as f32;
  let u = (uv.x * 3.0) as f32;
  let v = (1.0 - uv.y) as f32;
  
  let mut column = floorf(u * world_width) as u32;
  let in_even_column = column % 2 == 0;
  let offset = if in_even_column { 0.0 } else { 0.5 };
  let mut row = floorf(0.5 * v * world_width - offset) as u32;

  let x_in_square = u * world_width - (column as f32);
  let y_in_square = 0.5 * v * world_width - offset - (row as f32);
  let possibly_out_of_hex = x_in_square > 0.66667;
  if possibly_out_of_hex {
    let parameter_upper = y_in_square + 1.5 * x_in_square - 2.0;
    let parameter_lower = y_in_square - 1.5 * x_in_square + 1.0;
    if parameter_upper > 0.0 || parameter_lower < 0.0 {
      column += 1;
    }
    if parameter_upper > 0.0 && !in_even_column {
      row += 1;
    }
    if parameter_lower < 0.0 && in_even_column {
      row -= 1;
    }
  }

  let hexel_x: u32 = (column / 2) - row;
  let hexel_y: u32 = row * 2 + ((if column % 2 == 0 { 0 } else { 1 }) as u32);

  if hexel_x > (world_width as u32) || hexel_y > (world_width as u32) {
    *output = vec4(0.0, 0.0, 0.0, 0.0);
  } else {
    let coords = vec2(((hexel_x as f32) + 0.5) / (world_width as f32), ((hexel_y as f32) + 0.5) / (world_width as f32));
    *output = material_color_texture.sample(*material_sampler, coords)
  }
}

#[spirv(compute(threads(32, 32)))]
pub fn main(
  #[spirv(global_invocation_id)] global_id: UVec3,
  #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] input: &[Cell],
  #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] output: &mut [Cell],
) {
  let idx = get_index(global_id);
  if idx < input.len() {
    update_cell(input, output, global_id);
  }
}

fn get_index(global_id: UVec3) -> usize {
  let x = global_id.x as usize;
  let y = global_id.y as usize;
  y * WORLD_WIDTH + x
}

fn update_cell(input: &[Cell], output: &mut [Cell], global_id: UVec3) {
  let new_state = get_new_state(input, global_id);
  set_cell_at(output, global_id, new_state);
}

fn set_cell_at(buf: &mut [Cell], global_id: UVec3, cell: Cell) {
  buf[get_index(global_id)] = cell;
}

fn icell_at(buf: &[Cell], global_id: IVec3) -> Cell {
  if global_id.x < 0 || global_id.y < 0 {
    Cell::Dead
  } else {
    cell_at(buf, global_id.as_uvec3())
  }
}

fn cell_at(buf: &[Cell], global_id: UVec3) -> Cell {
  buf[get_index(global_id)]
}

fn get_new_state(buf: &[Cell], global_id: UVec3) -> Cell {
  match cell_at(buf, global_id) {
    Cell::Alive => {
      match count_living_neighbors(buf, global_id) {
        2..=3 => Cell::Alive,
        _ => Cell::Dead,
      }
    }
    Cell::Dead => {
      match count_living_neighbors(buf, global_id) {
        3 => Cell::Alive,
        _ => Cell::Dead,
      }
    }
  } 
}

fn count_living_neighbors(buf: &[Cell], global_id: UVec3) -> i32 {
  let mut num_living_neighbors = 0;
  for x1 in -1..=1 {
    for y1 in -1..=1 {
      if icell_at(buf, global_id.as_ivec3() + IVec3::new(x1, y1, 0)) == Cell::Alive && !(x1 == 0 && y1 == 0) {
        num_living_neighbors += 1;
      }
    }
  }
  num_living_neighbors
}
