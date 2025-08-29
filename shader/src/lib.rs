#![allow(unexpected_cfgs)]
#![cfg_attr(target_arch = "spirv", no_std)]

use biosim_core::{hex_grid::{uv_to_hexel_coord, uv_to_rect_grid_coord}, world::{get_index, Cell, WorldCoord, WorldCursor}, WORLD_WIDTH};
use spirv_std::{glam::{vec4, UVec3, Vec2, Vec3, Vec4}, spirv};

#[spirv(fragment)]
pub fn fragment(
    _: Vec3,
    _: Vec3,
    uv: Vec2,
    #[spirv(storage_buffer, descriptor_set = 2, binding = 0)] cells: &[Cell], 
    output: &mut Vec4
) {
    if cfg!(feature = "rect_grid") {
        rect_grid(uv, cells, output);
    } else {
        hex_grid(uv, cells, output);
    }
}

fn rect_grid(uv: Vec2, cells: &[Cell], output: &mut Vec4) {
  let coord = uv_to_rect_grid_coord(uv.x, 1.0 - uv.y);
  *output = cell_to_color(cells[get_index(coord)]);
}

fn hex_grid(uv: Vec2, cells: &[Cell], output: &mut Vec4) {
    let coord = uv_to_hexel_coord(uv.x, 1.0 - uv.y);
    let WorldCoord { x: hexel_x, y: hexel_y } = coord.clone();

    if hexel_x > WORLD_WIDTH || hexel_y > WORLD_WIDTH {
        *output = vec4(0.0, 0.0, 0.0, 0.0);
    } else {
        *output = cell_to_color(cells[get_index(coord)]);
    }
}

fn cell_to_color(cell: Cell) -> Vec4 {
    match cell {
        Cell::Alive => vec4(0.0, 0.0, 0.0, 1.0),
        Cell::Dead => vec4(1.0, 1.0, 1.0, 1.0),
    }
}

#[spirv(compute(threads(32, 32)))]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] input: &[Cell],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] output: &mut [Cell],
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
