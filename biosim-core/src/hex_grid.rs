use libm::floorf;

use crate::{world::WorldCoord, WORLD_WIDTH, WORLD_WIDTH_MULTIPLER};

pub fn uv_to_hexel_coord(u: f32, v: f32) -> WorldCoord {
    let world_width = WORLD_WIDTH as f32;
    let u = u * (WORLD_WIDTH_MULTIPLER / 2.0);
    let v = v;
    
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

    WorldCoord { x: hexel_x as usize, y: hexel_y as usize }
}

pub fn uv_to_rect_grid_coord(u: f32, v: f32) -> WorldCoord {
    let x = (u * WORLD_WIDTH as f32).clamp(0.0, WORLD_WIDTH as f32 - 1.0) as usize;
    let y = (v * WORLD_WIDTH as f32).clamp(0.0, WORLD_WIDTH as f32 - 1.0) as usize;
    WorldCoord { x, y }
}

pub fn world_space_to_uv(x: f32, y: f32) -> (f32, f32) {
    let world_width = WORLD_WIDTH as f32;
    ((x + (world_width * WORLD_WIDTH_MULTIPLER * 0.5)) / (world_width * WORLD_WIDTH_MULTIPLER), (y + (world_width * 0.5)) / world_width)
}
