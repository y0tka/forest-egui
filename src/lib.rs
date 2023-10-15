use rand::prelude::*;
use rand::seq::SliceRandom;
use rand_chacha::ChaCha8Rng;

use std::iter::repeat_with;

use strum_macros::EnumIter;

#[derive(Debug, Clone)]
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

    pub fn step(&mut self) -> Self {
        if self.age > 15 && self.cell_type == CellType::Flame {
            return Self {
                age: 0,
                cell_type: CellType::Empty,
                propagation: 0,
            };
        }
        Self {
            age: self.age + 1,
            cell_type: self.cell_type.clone(),
            propagation: self.propagation,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, EnumIter)]
pub enum CellType {
    Empty,
    Grass,
    Tree,
    Flame,
}

pub fn get_field_step(field: &[Cell]) -> Vec<Cell> {
    let mut return_field = field.to_vec();
    for cell in return_field.iter_mut() {
        *cell = cell.step();
    }
    return_field = propagate(&return_field);
    return_field
}

pub fn get_empty_field(size: usize) -> Vec<Cell> {
    let field: Vec<Cell> = repeat_with(Cell::new).take(size * size).collect();
    field
}

pub fn get_random_field(
    size: usize,
    grass_count: usize,
    tree_count: usize,
    flame_count: usize,
) -> Vec<Cell> {
    let mut field = get_empty_field(size);
    field = rnd_fill_empty(&field, grass_count, CellType::Grass);
    field = rnd_fill_empty(&field, tree_count, CellType::Tree);
    field = rnd_fill_empty(&field, flame_count, CellType::Flame);
    field
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

pub fn rnd_fill_empty(field: &[Cell], count: usize, cell_type: CellType) -> Vec<Cell> {
    let mut rng = ChaCha8Rng::seed_from_u64(0);
    let mut return_field = field.to_vec();
    let side = f32::sqrt(return_field.len() as f32).floor() as usize;
    let mut to_fill: usize = count;
    while to_fill > 0 {
        let x = rng.gen_range(0..side);
        let y = rng.gen_range(0..side);
        match cartesian_to_linear(x, y, &return_field) {
            Ok(coord) => {
                if CellType::Empty == return_field[coord].cell_type {
                    return_field[coord].cell_type = cell_type.clone();
                    return_field[coord].age = rng.gen_range(0..10);
                    to_fill -= 1;
                }
            }
            Err(_) => panic!("OH SHIT *dies from cringe*"),
        }
    }
    return_field
}

pub fn propagate(field: &Vec<Cell>) -> Vec<Cell> {
    let mut return_field: Vec<Cell> = field.clone().to_vec();
    let mut rng = ChaCha8Rng::seed_from_u64(0);
    for (index, ele) in field.iter().enumerate() {
        if ele.age < 8 {
            continue;
        }
        if ele.propagation == 1 {
            let x_p: f32 = rng.gen();
            let y_p: f32 = rng.gen();
            let offset: [isize; 2] = if y_p > x_p {
                [0, *[-1, 1].choose(&mut rng).unwrap()]
            } else {
                [*[-1, 1].choose(&mut rng).unwrap(), 0]
            };
            let (mut x, mut y) = linear_to_cartesian(index, field).unwrap();
            match x.checked_add_signed(offset[0]) {
                Some(res) => x = res,
                None => continue,
            }
            match y.checked_add_signed(offset[1]) {
                Some(res) => y = res,
                None => continue,
            }
            match cartesian_to_linear(x, y, field) {
                Ok(coord) => match ele.cell_type {
                    CellType::Grass | CellType::Tree => {
                        if CellType::Empty == return_field[coord].cell_type {
                            return_field[coord].cell_type = ele.cell_type.clone();
                            return_field[coord].age = 0;
                            return_field[coord].propagation = 1;
                        }
                    }
                    CellType::Flame => match return_field[coord].cell_type {
                        CellType::Grass | CellType::Tree => {
                            return_field[coord].cell_type = ele.cell_type.clone();
                            return_field[coord].age = 0;
                            return_field[coord].propagation = 1;
                        }
                        _ => (),
                    },
                    _ => (),
                },
                Err(_) => continue,
            }
        }
    }
    return_field
}
