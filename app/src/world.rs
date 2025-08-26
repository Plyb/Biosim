use biosim_core::{world::{get_index, Cell, WorldCoord, WorldCursor}, WORLD_WIDTH};

pub fn new_random() -> Vec<Cell> {
  let mut cells: Vec<Cell> = Vec::with_capacity(WORLD_WIDTH * WORLD_WIDTH);
  for _ in 0..WORLD_WIDTH {
    for _ in 0..WORLD_WIDTH {
      cells.push(rand::random());
    }
  }
  cells
}

pub fn tick(cells: &Vec<Cell>) -> Vec<Cell> {
  let mut new_cells : Vec<Cell> = vec![Cell::Dead; WORLD_WIDTH * WORLD_WIDTH];

  for x in 0..WORLD_WIDTH {
    for y in 0..WORLD_WIDTH {
      let coord = WorldCoord { x, y };
      let cursor = WorldCursor::new(cells.as_slice().try_into().unwrap(), coord);
      new_cells[get_index(coord)] = cursor.get_new_state();
    }
  }
  new_cells
}
