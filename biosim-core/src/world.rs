use rand::{distributions::{Distribution, Standard}, Rng};
use serde::Serialize;

#[derive(PartialEq, Clone, Serialize, Debug)]
pub enum Cell {
    Dead,
    Alive,
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
