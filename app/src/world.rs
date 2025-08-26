use biosim_core::{world::Cell, WORLD_WIDTH};
use serde::Serialize;
use serde_big_array::Array;


pub fn new_random() -> Vec<Cell> {
  let mut cells: Vec<Cell> = Vec::with_capacity(WORLD_WIDTH * WORLD_WIDTH);
  for x in 0..WORLD_WIDTH {
    for y in 0..WORLD_WIDTH {
      cells.push(rand::random());
    }
  }
  cells
}

pub fn tick(cells: &Vec<Cell>) -> Vec<Cell> {
  let mut new_cells : Vec<Cell> = vec![Cell::Dead; WORLD_WIDTH * WORLD_WIDTH];

  for x in 0..WORLD_WIDTH {
    for y in 0..WORLD_WIDTH {
      new_cells[y * WORLD_WIDTH + x] = get_new_state(cells, x, y);
    }
  }
  new_cells
}

fn get_new_state(cells: &Vec<Cell>, x: usize, y: usize) -> Cell {
  match cells[y * WORLD_WIDTH + x] {
    Cell::Alive => {
      match count_living_neighbors(cells, x as i32, y as i32) {
        2..=3 => Cell::Alive,
        _ => Cell::Dead,
      }
    }
    Cell::Dead => {
      match count_living_neighbors(cells, x as i32, y as i32) {
        3 => Cell::Alive,
        _ => Cell::Dead,
      }
    }
    Cell::Blah => Cell::Alive
  } 
}

fn count_living_neighbors(cells: &Vec<Cell>, x: i32, y: i32) -> i32 {
  let mut num_living_neighbors = 0;
  for x1 in -1..=1 {
    for y1 in -1..=1 {
      if get_cell(cells, x + x1, y + y1) == Cell::Alive && !(x1 == 0 && y1 == 0) {
        num_living_neighbors += 1;
      }
    }
  }
  num_living_neighbors
}

fn get_cell(cells: &Vec<Cell>, x: i32, y: i32) -> Cell {
  if x < 0 || y < 0 || x >= WORLD_WIDTH.try_into().unwrap() || y >= WORLD_WIDTH.try_into().unwrap() {
    Cell::Dead
  } else {
    cells[y as usize * WORLD_WIDTH + x as usize].clone()
  }
}
