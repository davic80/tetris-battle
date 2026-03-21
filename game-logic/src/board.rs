use crate::piece::Piece;
use serde::{Deserialize, Serialize};

pub const WIDTH: usize = 10;
pub const HEIGHT: usize = 20;
pub const HIDDEN: usize = 4; // hidden rows above visible area

#[derive(Clone)]
pub struct Board {
    // (HEIGHT + HIDDEN) rows × WIDTH cols; 0 = empty, 1-7 = piece color
    pub cells: Vec<Vec<u8>>,
}

impl Board {
    pub fn new() -> Self {
        Board {
            cells: vec![vec![0u8; WIDTH]; HEIGHT + HIDDEN],
        }
    }

    pub fn is_valid(&self, piece: &Piece) -> bool {
        for [cx, cy] in piece.absolute_cells() {
            if cx < 0 || cx >= WIDTH as i32 {
                return false;
            }
            if cy >= (HEIGHT + HIDDEN) as i32 {
                return false;
            }
            if cy >= 0 {
                if self.cells[cy as usize][cx as usize] != 0 {
                    return false;
                }
            }
        }
        true
    }

    pub fn lock_piece(&mut self, piece: &Piece) {
        let color = piece.piece_type as u8 + 1;
        for [cx, cy] in piece.absolute_cells() {
            if cy >= 0 && cy < (HEIGHT + HIDDEN) as i32 && cx >= 0 && cx < WIDTH as i32 {
                self.cells[cy as usize][cx as usize] = color;
            }
        }
    }

    /// Clears full lines and returns count of lines cleared
    pub fn clear_lines(&mut self) -> u32 {
        let total = HEIGHT + HIDDEN;
        let mut cleared = 0u32;
        let mut new_cells: Vec<Vec<u8>> = Vec::with_capacity(total);

        for row in &self.cells {
            if row.iter().all(|&c| c != 0) {
                cleared += 1;
            } else {
                new_cells.push(row.clone());
            }
        }
        // Add empty rows at top
        while new_cells.len() < total {
            new_cells.insert(0, vec![0u8; WIDTH]);
        }
        self.cells = new_cells;
        cleared
    }

    /// Add garbage lines at the bottom with a random hole
    pub fn add_garbage(&mut self, count: u32) {
        let hole = (js_sys::Math::random() * WIDTH as f64) as usize;
        for _ in 0..count {
            // Remove top row
            self.cells.remove(0);
            // Add garbage row at bottom
            let mut row = vec![8u8; WIDTH]; // color 8 = garbage (gray)
            row[hole] = 0;
            self.cells.push(row);
        }
    }

    /// Returns only visible rows (without hidden buffer)
    pub fn visible_cells(&self) -> &[Vec<u8>] {
        &self.cells[HIDDEN..]
    }
}
