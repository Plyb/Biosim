#![allow(unexpected_cfgs)]
#![cfg_attr(target_arch = "spirv", no_std)]

use biosim_core::{world::{get_index, Cell, WorldCoord, WorldCursor}, WORLD_WIDTH};
use libm::floorf;
use spirv_std::{glam::{vec2, vec4, UVec3, Vec2, Vec3, Vec4}, image::Image2d, spirv, Sampler};

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

#[spirv(compute(threads(1, 1)))]
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
