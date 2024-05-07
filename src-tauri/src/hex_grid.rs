use std::f32::consts::PI;

use bevy::render::{color::Color, mesh::{Indices, Mesh, PrimitiveTopology}, render_asset::RenderAssetUsages};

use crate::world::WORLD_WIDTH;

const NUM_VERTS_PER_HEX: u32 = 6;
pub fn build_hex_grid_mesh() -> Mesh {
  let mut mesh = Mesh::new(
    PrimitiveTopology::TriangleList,
    RenderAssetUsages::all()
  );
  mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, get_positions());
  mesh.insert_indices(Indices::U32(get_indices()));
  mesh.insert_attribute(
    Mesh::ATTRIBUTE_COLOR, 
    vec![
      Color::WHITE.as_linear_rgba_f32();
      WORLD_WIDTH * WORLD_WIDTH * NUM_VERTS_PER_HEX as usize
    ]
  );

  mesh
}

fn get_positions() -> Vec<[f32; 3]> {
  let mut positions: Vec<[f32; 3]> = vec![];

  let delta_y = (3.0f32).sqrt() / 2.;
  for x in 0..WORLD_WIDTH {
    for y in 0..WORLD_WIDTH {
      let x_center = (3. * x as f32) + (3. * (y as f32 / 2.));
      let y_center = delta_y * y as f32;

      for v in 0..6 {
        positions.push(get_hex_vertex_pos(x_center, y_center, v))
      }
    }
  }

  positions
}

fn get_hex_vertex_pos(x_center: f32, y_center: f32, vertex_index: usize) -> [f32; 3] {
  const ANGLE_MULTIPLIER: f32 = PI / 3.;

  let angle = ANGLE_MULTIPLIER * vertex_index as f32;
  [angle.cos() + x_center, angle.sin() + y_center, 0.]
}

fn get_indices() -> Vec<u32> {
  let mut indices: Vec<u32> = vec![];

  for x in 0..WORLD_WIDTH {
    for y in 0..WORLD_WIDTH {
      let offset = (x as u32 + (y * WORLD_WIDTH) as u32) * NUM_VERTS_PER_HEX;
      indices.extend_from_slice(&[
        offset + 0, offset + 1, offset + 5,
        offset + 1, offset + 2, offset + 5,
        offset + 2, offset + 4, offset + 5,
        offset + 2, offset + 3, offset + 4,
      ]);
    }
  }

  indices
}