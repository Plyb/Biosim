#![allow(unexpected_cfgs)]
#![cfg_attr(target_arch = "spirv", no_std)]

use biosim_core::{hex_grid::uv_to_hexel_coord, world::{get_index, Cell, WorldCoord, WorldCursor}, WORLD_WIDTH};
use spirv_std::{glam::{vec2, vec4, UVec3, Vec2, Vec3, Vec4}, image::Image2d, spirv, Sampler};

#[spirv(fragment)]
pub fn fragment(
    _: Vec3,
    _: Vec3,
    uv: Vec2,
    #[spirv(descriptor_set = 2, binding = 0)] material_color_texture: &Image2d,
    #[spirv(descriptor_set = 2, binding = 1)] material_sampler: &Sampler,
    #[spirv(storage_buffer, descriptor_set = 2, binding = 2)] cells: &[Cell], 
    output: &mut Vec4
) {
    if cfg!(feature = "rect_grid") {
        rect_grid(uv, material_color_texture, material_sampler, output);
    } else {
        hex_grid(uv, material_color_texture, material_sampler, cells, output);
    }
}

fn rect_grid(uv: Vec2, material_color_texture: &Image2d, material_sampler: &Sampler, output: &mut Vec4) {
  let world_width = WORLD_WIDTH as f32;
  let uv = Vec2 { x: uv.x, y: 1.0 - uv.y };
  *output = material_color_texture.sample(*material_sampler, ((uv * world_width).floor() + 0.5) / world_width)
}

fn hex_grid(uv: Vec2, material_color_texture: &Image2d, material_sampler: &Sampler, cells: &[Cell], output: &mut Vec4) {
    let world_width = WORLD_WIDTH as f32;
    let coord = uv_to_hexel_coord(uv.x, 1.0 - uv.y);
    let WorldCoord { x: hexel_x, y: hexel_y } = coord.clone();

    if hexel_x > WORLD_WIDTH || hexel_y > WORLD_WIDTH {
        *output = vec4(0.0, 0.0, 0.0, 0.0);
    } else {
        // let coords = vec2(((hexel_x as f32) + 0.5) / (world_width as f32), ((hexel_y as f32) + 0.5) / (world_width as f32));
        *output = match cells[get_index(coord)] {
            Cell::Alive => vec4(0.0, 0.0, 0.0, 1.0),
            Cell::Dead => vec4(1.0, 1.0, 1.0, 1.0),
        };// material_color_texture.sample(*material_sampler, coords)
    }
}

#[spirv(compute(threads(32, 32)))]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] input: &[Cell],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] output: &mut [Cell],
) {
    let coord = WorldCoord { x: global_id.x as usize, y: global_id.y as usize };
    let idx = get_index(coord);
    if idx < input.len() {
        update_cell(input, output, coord);
    }
}

fn update_cell(input: &[Cell], output: &mut [Cell], coord: WorldCoord) {
    let cursor = WorldCursor::new(input, coord);
    
    let new_state = cursor.get_new_state();
    set_cell_at(output, coord, new_state);
}

fn set_cell_at(buf: &mut [Cell], coord: WorldCoord, cell: Cell) {
    buf[get_index(coord)] = cell;
}
