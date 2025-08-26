use bytemuck::{Pod, Zeroable};
use rand::{distributions::{Distribution, Standard}, Rng};
use serde::Serialize;

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
