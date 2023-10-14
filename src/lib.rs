use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Cell {
    pub age: usize,
    pub cell_type: CellType,
    pub propagation: u8,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            age: 0,
            cell_type: CellType::Empty,
            propagation: 1,
        }
    }
}

impl Cell {
    pub fn new() -> Cell {
        Cell {
            age: 0,
            cell_type: CellType::Empty,
            propagation: 1,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, EnumIter, Serialize, Deserialize)]
pub enum CellType {
    Empty,
    Grass,
    Tree,
    Flame,
}

pub fn cartesian_to_linear(y: usize, x: usize, field: &Vec<Cell>) -> Result<usize, &'static str> {
    let area = field.len();
    let side = f32::sqrt(area as f32).floor() as usize;
    if y >= side || x >= side {
        return Err("Out of Bounds (C2L)");
    }
    let res = x * side + y;
    Ok(res)
}

pub fn linear_to_cartesian(
    position: usize,
    field: &Vec<Cell>,
) -> Result<(usize, usize), &'static str> {
    let side = f32::sqrt(field.len() as f32).floor() as usize;
    if position >= field.len() {
        return Err("Out of Bounds (L2C)");
    }
    Ok((position % side, position / side))
}
