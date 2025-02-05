use biosim_core::{world::Cell, WORLD_WIDTH};
use serde::Serialize;
use serde_big_array::Array;

#[derive(Default, Serialize, Clone, Debug)]
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
