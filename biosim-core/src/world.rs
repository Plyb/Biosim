use core::ops;

use bytemuck::{Pod, Zeroable};
use rand::{distributions::{Distribution, Standard}, Rng};
use serde::Serialize;

use crate::WORLD_WIDTH;

use crate::util::DOption;

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

#[derive(Copy, Clone, Default, Debug)]
pub struct WorldCoord {
    pub x: usize,
    pub y: usize,
}

pub struct WorldOffset {
    pub x: i32,
    pub y: i32,
}

impl ops::Add<WorldOffset> for WorldCoord {
    type Output = DOption<WorldCoord>;

    fn add(self, rhs: WorldOffset) -> Self::Output {
        let x = self.x as i32 + rhs.x;
        let y = self.y as i32 + rhs.y;
        if x < 0 || y < 0 || x as usize >= WORLD_WIDTH || y as usize >= WORLD_WIDTH {
            DOption::none()
        } else {
            DOption::some(WorldCoord { x: x as usize, y: y as usize })
        }
    }
}

impl WorldCoord {
    pub fn add_clamped(&self, offset: WorldOffset) -> WorldCoord {
        let x = (self.x as i32 + offset.x).clamp(0, WORLD_WIDTH as i32 - 1) as usize;
        let y = (self.y as i32 + offset.y).clamp(0, WORLD_WIDTH as i32 - 1) as usize;
        WorldCoord { x, y }
    }
}

impl WorldOffset {
    fn zero() -> WorldOffset {
        WorldOffset { x: 0, y: 0 }
    }
}

pub struct WorldCursor<'a> {
    coord: WorldCoord,
    cells: &'a [Cell],
}

impl<'a> WorldCursor<'a> {
    pub fn new(cells: &'a [Cell], coord: WorldCoord) -> WorldCursor<'a> {
        WorldCursor { coord, cells }
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

    fn get_cell_at_offset(&self, offset: WorldOffset) -> Cell {
            match self.coord + offset {
                DOption(true, coord) => self.get_cell_at_coord(coord),
                DOption(false, _) => Cell::Dead
            }
        }
    
    fn get_cell_at_coord(&self, coord: WorldCoord) -> Cell {
        self.cells[
                get_index(coord)
            ]
    }

    fn get_cell(&self) -> Cell {
        self.get_cell_at_offset(WorldOffset::zero())
    }

    fn count_living_neighbors(&self) -> i32 {
        let mut num_living_neighbors = 0;
        for x in -1..=1 {
            for y in -1..=1 {
                if self.get_cell_at_offset(WorldOffset { x, y }) == Cell::Alive && !(x == 0 && y == 0) {
                    num_living_neighbors += 1;
                }
            }
        }
        num_living_neighbors
    }
}

pub fn get_index(coord: WorldCoord) -> usize {
    coord.y * WORLD_WIDTH + coord.x
}

