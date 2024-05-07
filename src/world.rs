use std::fmt::Debug;
use rand::{Rng, distributions::{Standard, Distribution}};
use serde::Serialize;
use serde_big_array::Array;

pub const WORLD_WIDTH: usize = 32;

#[derive(PartialEq, Clone, Serialize)]
pub enum Cell {
    Dead,
    Alive,
}

impl Default for Cell {
    fn default() -> Self {
        Cell::Dead
    }
}

impl Debug for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
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

impl ToString for Cell {
    fn to_string(&self) -> String {
        match self {
            Cell::Alive => "#".into(),
            Cell::Dead => "-".into(),
        }
    }
}

#[derive(Default, Serialize, Clone)]
pub struct World {
    pub cells: Vec<Array<Cell, WORLD_WIDTH>>}

impl World {
    pub fn new_random() -> World {
        let mut cells: Vec<Array<Cell, WORLD_WIDTH>> = Vec::with_capacity(WORLD_WIDTH);
        for x in 0..WORLD_WIDTH {
            cells.push(Array([Cell::Dead; WORLD_WIDTH]));
            for y in 0..WORLD_WIDTH {
                cells[x][y] = rand::random();
            }
        }
        World { cells }
    }

    pub fn tick(&self) -> World {
        let mut new_world : World = Default::default();
        new_world.cells = Vec::with_capacity(WORLD_WIDTH);

        for x in 0..WORLD_WIDTH {
            new_world.cells.push(Array([Cell::Dead; WORLD_WIDTH]));
            for y in 0..WORLD_WIDTH {
                new_world.cells[x][y] = self.get_new_state(x, y);
            }
        }
        new_world
    }

    fn get_new_state(&self, x: usize, y: usize) -> Cell {
        match self.cells[x][y] {
            Cell::Alive => {
                match self.count_living_neighbors(x as i32, y as i32) {
                    2..=3 => Cell::Alive,
                    _ => Cell::Dead,
                }
            }
            Cell::Dead => {
                match self.count_living_neighbors(x as i32, y as i32) {
                    3 => Cell::Alive,
                    _ => Cell::Dead,
                }
            }
        }
        
    }

    fn count_living_neighbors(&self, x: i32, y: i32) -> i32 {
        let mut num_living_neighbors = 0;
        for x1 in -1..=1 {
            for y1 in -1..=1 {
                if self.get_cell(x + x1, y + y1) == Cell::Alive && !(x1 == 0 && y1 == 0) {
                    num_living_neighbors += 1;
                }
            }
        }
        num_living_neighbors
    }

    fn get_cell(&self, x: i32, y: i32) -> Cell {
        if x < 0 || y < 0 || x >= WORLD_WIDTH.try_into().unwrap() || y >= WORLD_WIDTH.try_into().unwrap() {
            Cell::Dead
        } else {
            self.cells[x as usize][y as usize].clone()
        }
    }
}

impl Debug for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = self.cells.iter().map(|row| {
            row.map(|cell| {
                cell.to_string()
            }).join("")
        }).collect::<Vec<_>>().join("\n");

        write!(f, "{}", string)
    }
}
