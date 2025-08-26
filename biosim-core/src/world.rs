use bytemuck::{Pod, Zeroable};
use rand::{distributions::{Distribution, Standard}, Rng};
use serde::Serialize;

use crate::WORLD_WIDTH;

#[repr(u32)]
#[derive(PartialEq, Clone, Serialize, Debug)]
pub enum Cell {
  Dead = 0,
  Alive = 1,
}

unsafe impl Zeroable for Cell {
  fn zeroed() -> Self {
    Cell::Dead
  }
}

unsafe impl Pod for Cell {
  
}

impl Default for Cell {
  fn default() -> Self {
    Cell::Dead
  }
}

impl Distribution<Cell> for Standard {
  fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Cell {
    match rng.gen_range(0..=1) {
      0 => Cell::Dead,
      _ => Cell::Alive,
    }
  }
}

impl Copy for Cell {}

pub struct WorldCursor<'a> {
  x: usize,
  y: usize,
  cells: &'a [Cell; WORLD_WIDTH * WORLD_WIDTH]
}

impl<'a> WorldCursor<'a> {
  pub fn new(cells: &'a [Cell; WORLD_WIDTH * WORLD_WIDTH], x: usize, y: usize) -> WorldCursor<'a> {
    WorldCursor { x, y, cells }
  }

  pub fn get_new_state(&self) -> Cell {
    match self.get_cell() {
      Cell::Alive => {
        match self.count_living_neighbors() {
          2..=3 => Cell::Alive,
          _ => Cell::Dead,
        }
      }
      Cell::Dead => {
        match self.count_living_neighbors() {
          3 => Cell::Alive,
          _ => Cell::Dead,
        }
      }
    } 
  }

  fn get_cell_at(&self, x_offset: i32, y_offset: i32) -> Cell {
        if self.is_valid_offset(x_offset, y_offset) {
          self.get_cell_at_raw(x_offset, y_offset)
        } else {
          Cell::Dead
        }
    }
  
  fn get_cell_at_raw(&self, x_offset: i32, y_offset: i32) -> Cell {
    self.cells[
        get_index((self.x as i32 + x_offset) as usize, (self.y as i32 + y_offset) as usize)
      ]
  }

  fn get_cell(&self) -> Cell {
    self.get_cell_at(0, 0)
  }

  fn is_valid_offset(&self, x_offset: i32, y_offset: i32) -> bool {
    let x = self.x as i32 + x_offset;
    let y = self.y as i32 + y_offset;
    !(x < 0 || y < 0 || x as usize >= WORLD_WIDTH || y as usize >= WORLD_WIDTH)
  }

  fn count_living_neighbors(&self) -> i32 {
    let mut num_living_neighbors = 0;
    for x1 in -1..=1 {
      for y1 in -1..=1 {
        if self.get_cell_at(x1, y1) == Cell::Alive && !(x1 == 0 && y1 == 0) {
          num_living_neighbors += 1;
        }
      }
    }
    num_living_neighbors
  }
}

pub fn get_index(x: usize, y: usize) -> usize {
  y * WORLD_WIDTH + x
}

