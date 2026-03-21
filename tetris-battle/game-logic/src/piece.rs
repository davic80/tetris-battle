use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[repr(u8)]
pub enum PieceType {
    I = 0,
    O = 1,
    T = 2,
    S = 3,
    Z = 4,
    J = 5,
    L = 6,
}

#[derive(Clone, Debug)]
pub struct Piece {
    pub piece_type: PieceType,
    pub rotation: u8,         // 0-3
    pub x: i32,               // column offset
    pub y: i32,               // row offset (0 = top of board including hidden)
    pub cells: Vec<[i32; 2]>, // relative [col, row] from (x,y)
}

impl Piece {
    pub fn new(pt: PieceType) -> Self {
        let cells = cells_for(pt, 0);
        Piece {
            piece_type: pt,
            rotation: 0,
            x: 3, // center-ish spawn
            y: 0, // spawn at top of board (hidden area)
            cells,
        }
    }

    pub fn absolute_cells(&self) -> Vec<[i32; 2]> {
        self.cells
            .iter()
            .map(|[cx, cy]| [cx + self.x, cy + self.y])
            .collect()
    }
}

/// Returns cells for a given piece type and rotation (relative coordinates)
pub fn cells_for(pt: PieceType, rot: u8) -> Vec<[i32; 2]> {
    // Standard Tetris Guideline piece definitions
    // Format: [col, row] relative offsets
    match pt {
        PieceType::I => match rot {
            0 => vec![[0, 1], [1, 1], [2, 1], [3, 1]],
            1 => vec![[2, 0], [2, 1], [2, 2], [2, 3]],
            2 => vec![[0, 2], [1, 2], [2, 2], [3, 2]],
            _ => vec![[1, 0], [1, 1], [1, 2], [1, 3]],
        },
        PieceType::O => match rot {
            _ => vec![[1, 0], [2, 0], [1, 1], [2, 1]],
        },
        PieceType::T => match rot {
            0 => vec![[1, 0], [0, 1], [1, 1], [2, 1]],
            1 => vec![[1, 0], [1, 1], [2, 1], [1, 2]],
            2 => vec![[0, 1], [1, 1], [2, 1], [1, 2]],
            _ => vec![[1, 0], [0, 1], [1, 1], [1, 2]],
        },
        PieceType::S => match rot {
            0 => vec![[1, 0], [2, 0], [0, 1], [1, 1]],
            1 => vec![[1, 0], [1, 1], [2, 1], [2, 2]],
            2 => vec![[1, 1], [2, 1], [0, 2], [1, 2]],
            _ => vec![[0, 0], [0, 1], [1, 1], [1, 2]],
        },
        PieceType::Z => match rot {
            0 => vec![[0, 0], [1, 0], [1, 1], [2, 1]],
            1 => vec![[2, 0], [1, 1], [2, 1], [1, 2]],
            2 => vec![[0, 1], [1, 1], [1, 2], [2, 2]],
            _ => vec![[1, 0], [0, 1], [1, 1], [0, 2]],
        },
        PieceType::J => match rot {
            0 => vec![[0, 0], [0, 1], [1, 1], [2, 1]],
            1 => vec![[1, 0], [2, 0], [1, 1], [1, 2]],
            2 => vec![[0, 1], [1, 1], [2, 1], [2, 2]],
            _ => vec![[1, 0], [1, 1], [0, 2], [1, 2]],
        },
        PieceType::L => match rot {
            0 => vec![[2, 0], [0, 1], [1, 1], [2, 1]],
            1 => vec![[1, 0], [1, 1], [1, 2], [2, 2]],
            2 => vec![[0, 1], [1, 1], [2, 1], [0, 2]],
            _ => vec![[0, 0], [1, 0], [1, 1], [1, 2]],
        },
    }
}

/// SRS wall kicks: returns list of (dx, dy) offsets to try
pub fn wall_kicks(pt: PieceType, from: u8, to: u8) -> Vec<(i32, i32)> {
    // Always try (0,0) first (no kick)
    if pt == PieceType::I {
        i_kicks(from, to)
    } else {
        jlstz_kicks(from, to)
    }
}

fn jlstz_kicks(from: u8, to: u8) -> Vec<(i32, i32)> {
    let table: [[(i32, i32); 5]; 8] = [
        // 0->R
        [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
        // R->0
        [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
        // R->2
        [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
        // 2->R
        [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
        // 2->L
        [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
        // L->2
        [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
        // L->0
        [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
        // 0->L
        [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
    ];
    let idx = match (from, to) {
        (0, 1) => 0,
        (1, 0) => 1,
        (1, 2) => 2,
        (2, 1) => 3,
        (2, 3) => 4,
        (3, 2) => 5,
        (3, 0) => 6,
        (0, 3) => 7,
        _ => 0,
    };
    table[idx].to_vec()
}

fn i_kicks(from: u8, to: u8) -> Vec<(i32, i32)> {
    let table: [[(i32, i32); 5]; 8] = [
        // 0->R
        [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
        // R->0
        [(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
        // R->2
        [(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
        // 2->R
        [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
        // 2->L
        [(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
        // L->2
        [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
        // L->0
        [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
        // 0->L
        [(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
    ];
    let idx = match (from, to) {
        (0, 1) => 0,
        (1, 0) => 1,
        (1, 2) => 2,
        (2, 1) => 3,
        (2, 3) => 4,
        (3, 2) => 5,
        (3, 0) => 6,
        (0, 3) => 7,
        _ => 0,
    };
    table[idx].to_vec()
}
