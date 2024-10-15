use core::fmt;

use bevy::prelude::*;

use crate::{CellState, GridComponent};

#[derive(Resource, Copy, Clone, PartialEq, Eq, Debug)]
pub struct GridSize {
    pub rows: usize,
    pub cols: usize,
}

pub struct Grid {
    pub grid_size: GridSize,
    grid: Vec<Vec<CellState>>,
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{} rows and {} cols",
            &self.grid_size.rows, &self.grid_size.cols
        )?;
        for row in &self.grid {
            for state in row {
                write!(f, "{}", state)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Grid {
    pub fn from_puzzle_string(str: String) -> Self {
        let mut lines = str.lines();
        lines.next();
        let mut numbers = lines
            .next()
            .unwrap()
            .split(",")
            .map(|n| n.parse::<usize>().unwrap());
        let grid_size = GridSize {
            cols: numbers.next().unwrap(),
            rows: numbers.next().unwrap(),
        };
        let mut grid = vec![vec![CellState::Blank; grid_size.cols]; grid_size.rows];
        let lines = lines.skip(2);
        for line in lines {
            let mut numbers = line.split(",").map(|n| n.parse::<i8>().unwrap());
            let size = numbers.next().unwrap();
            let row = (numbers.next().unwrap() - 1) as usize;
            let col = (numbers.next().unwrap() - 1) as usize;
            grid[row][col] = CellState::Value(size);
        }
        Grid { grid_size, grid }
    }

    pub fn from_solution_string(str: String) -> Self {
        let mut grid = Vec::new();
        for line in str.lines() {
            let mut row = Vec::new();
            for c in line.chars() {
                row.push(if c.is_digit(10) {
                    CellState::Value(c.to_digit(10).unwrap() as i8)
                } else if c == 'x' {
                    CellState::River
                } else {
                    CellState::Island
                });
            }
            grid.push(row);
        }
        let grid_size = GridSize {
            rows: grid.len(),
            cols: grid.get(0).unwrap().len(),
        };
        Grid { grid_size, grid }
    }

    pub fn get(&self, row: usize, col: usize) -> CellState {
        *self
            .grid
            .get(row.clamp(0, self.grid_size.rows as usize))
            .unwrap()
            .get(col.clamp(0, self.grid_size.cols as usize))
            .unwrap()
    }

    pub fn set(&mut self, location: &GridComponent, value: CellState) {
        self.grid[location.row][location.col] = value;
    }

    pub fn check(&self, solution: &Grid) -> bool {
        dbg!("size: self: {} sol: {}", self.grid_size, solution.grid_size);
        if self.grid_size != solution.grid_size {
            return false;
        }
        for row in 0..self.grid_size.rows {
            for col in 0..self.grid_size.cols {
                if !self.get(row, col).is_same(solution.get(row, col)) {
                    dbg!(
                        "val at ({}, {}): self: {} sol: {}",
                        row,
                        col,
                        self.get(row, col),
                        solution.get(row, col)
                    );
                    return false;
                }
            }
        }
        true
    }
}

// grid = vec![vec!;
