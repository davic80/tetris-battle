use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

mod board;
mod garbage;
mod piece;
mod scoring;

pub use board::Board;
pub use piece::{Piece, PieceType};
pub use scoring::Scoring;

#[wasm_bindgen]
pub struct GameState {
    board: Board,
    current_piece: Piece,
    next_pieces: Vec<PieceType>,
    bag: Vec<PieceType>,
    scoring: Scoring,
    game_over: bool,
    pending_garbage: u32,
    next_count: u8,
    pending_attack: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GameSnapshot {
    pub board: Vec<Vec<u8>>,
    pub current: PieceSnapshot,
    pub next: Vec<PieceSnapshot>,
    pub ghost_y: i32,
    pub score: u32,
    pub level: u32,
    pub lines: u32,
    pub game_over: bool,
    pub pending_garbage: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PieceSnapshot {
    pub cells: Vec<[i32; 2]>,
    pub color: u8,
}

#[wasm_bindgen]
impl GameState {
    #[wasm_bindgen(constructor)]
    pub fn new(next_count: u8) -> GameState {
        let mut bag = Self::new_bag();
        let first = bag.pop().unwrap();
        let mut state = GameState {
            board: Board::new(),
            current_piece: Piece::new(first),
            next_pieces: Vec::new(),
            bag,
            scoring: Scoring::new(),
            game_over: false,
            pending_garbage: 0,
            next_count: next_count.min(4),
            pending_attack: 0,
        };
        state.refill_next();
        state
    }

    fn new_bag() -> Vec<PieceType> {
        use PieceType::*;
        let mut bag = vec![I, O, T, S, Z, J, L];
        for i in (1..bag.len()).rev() {
            let j = (js_sys::Math::random() * (i as f64 + 1.0)) as usize;
            bag.swap(i, j);
        }
        bag
    }

    fn refill_next(&mut self) {
        while self.next_pieces.len() < self.next_count as usize + 1 {
            if self.bag.is_empty() {
                self.bag = Self::new_bag();
            }
            let pt = self.bag.pop().unwrap();
            self.next_pieces.push(pt);
        }
    }

    fn spawn_next(&mut self) {
        let pt = self.next_pieces.remove(0);
        self.refill_next();
        self.current_piece = Piece::new(pt);
        if !self.board.is_valid(&self.current_piece) {
            self.game_over = true;
        }
    }

    #[wasm_bindgen]
    pub fn move_left(&mut self) -> bool {
        let mut p = self.current_piece.clone();
        p.x -= 1;
        if self.board.is_valid(&p) {
            self.current_piece = p;
            true
        } else {
            false
        }
    }

    #[wasm_bindgen]
    pub fn move_right(&mut self) -> bool {
        let mut p = self.current_piece.clone();
        p.x += 1;
        if self.board.is_valid(&p) {
            self.current_piece = p;
            true
        } else {
            false
        }
    }

    #[wasm_bindgen]
    pub fn soft_drop(&mut self) -> bool {
        let mut p = self.current_piece.clone();
        p.y += 1;
        if self.board.is_valid(&p) {
            self.current_piece = p;
            self.scoring.add_soft_drop(1);
            true
        } else {
            false
        }
    }

    #[wasm_bindgen]
    pub fn hard_drop(&mut self) -> u32 {
        let mut dropped = 0u32;
        loop {
            let mut p = self.current_piece.clone();
            p.y += 1;
            if self.board.is_valid(&p) {
                self.current_piece = p;
                dropped += 1;
            } else {
                break;
            }
        }
        self.scoring.add_hard_drop(dropped);
        self.lock_piece();
        dropped
    }

    #[wasm_bindgen]
    pub fn rotate_cw(&mut self) -> bool {
        self.try_rotate(1)
    }

    #[wasm_bindgen]
    pub fn rotate_ccw(&mut self) -> bool {
        self.try_rotate(-1)
    }

    fn try_rotate(&mut self, dir: i32) -> bool {
        let mut p = self.current_piece.clone();
        let old_rot = p.rotation;
        p.rotation = ((p.rotation as i32 + dir).rem_euclid(4)) as u8;
        p.cells = piece::cells_for(p.piece_type, p.rotation);
        let kicks = piece::wall_kicks(p.piece_type, old_rot, p.rotation);
        for (dx, dy) in kicks {
            let mut pk = p.clone();
            pk.x += dx;
            pk.y += dy;
            if self.board.is_valid(&pk) {
                self.current_piece = pk;
                return true;
            }
        }
        false
    }

    fn lock_piece(&mut self) {
        self.board.lock_piece(&self.current_piece);
        let lines_cleared = self.board.clear_lines();

        let attack = self.scoring.register_clear(lines_cleared);

        // Garbage cancellation: cleared lines cancel pending garbage
        if lines_cleared > 0 && self.pending_garbage > 0 {
            let cancelled = lines_cleared.min(self.pending_garbage);
            self.pending_garbage = self.pending_garbage.saturating_sub(cancelled);
        }

        // Apply remaining garbage
        if self.pending_garbage > 0 {
            let g = self.pending_garbage;
            self.board.add_garbage(g);
            self.pending_garbage = 0;
        }

        // Queue attack to send
        if attack > 0 {
            self.pending_attack += attack;
        }

        self.spawn_next();
    }

    /// Gravity tick: returns attack lines to send (0 if none)
    #[wasm_bindgen]
    pub fn gravity_tick(&mut self) -> u32 {
        if self.game_over {
            return 0;
        }
        if !self.soft_drop() {
            self.lock_piece();
        }
        let a = self.pending_attack;
        self.pending_attack = 0;
        a
    }

    /// Call after hard_drop to get attack lines generated
    #[wasm_bindgen]
    pub fn take_attack(&mut self) -> u32 {
        let a = self.pending_attack;
        self.pending_attack = 0;
        a
    }

    #[wasm_bindgen]
    pub fn receive_garbage(&mut self, lines: u32) {
        self.pending_garbage += lines;
    }

    #[wasm_bindgen]
    pub fn is_game_over(&self) -> bool {
        self.game_over
    }

    #[wasm_bindgen]
    pub fn get_score(&self) -> u32 {
        self.scoring.score
    }

    #[wasm_bindgen]
    pub fn get_level(&self) -> u32 {
        self.scoring.level
    }

    #[wasm_bindgen]
    pub fn get_lines(&self) -> u32 {
        self.scoring.lines
    }

    #[wasm_bindgen]
    pub fn get_gravity_ms(&self) -> u32 {
        self.scoring.gravity_ms()
    }

    /// Set level from server broadcast (shared global level)
    #[wasm_bindgen]
    pub fn set_level(&mut self, level: u32) {
        self.scoring.set_level(level);
    }

    #[wasm_bindgen]
    pub fn snapshot(&self) -> JsValue {
        let ghost_y = self.compute_ghost_y();
        let board = self.board.cells.clone();
        let current = PieceSnapshot {
            cells: self.current_piece.absolute_cells(),
            color: self.current_piece.piece_type as u8 + 1,
        };
        let next: Vec<PieceSnapshot> = self
            .next_pieces
            .iter()
            .take(self.next_count as usize)
            .map(|pt| {
                let p = Piece::new(*pt);
                PieceSnapshot {
                    cells: p.absolute_cells(),
                    color: *pt as u8 + 1,
                }
            })
            .collect();

        let snap = GameSnapshot {
            board,
            current,
            next,
            ghost_y,
            score: self.scoring.score,
            level: self.scoring.level,
            lines: self.scoring.lines,
            game_over: self.game_over,
            pending_garbage: self.pending_garbage,
        };
        serde_wasm_bindgen::to_value(&snap).unwrap()
    }

    fn compute_ghost_y(&self) -> i32 {
        let mut p = self.current_piece.clone();
        loop {
            let mut np = p.clone();
            np.y += 1;
            if self.board.is_valid(&np) {
                p = np;
            } else {
                break;
            }
        }
        p.y
    }

    #[wasm_bindgen]
    pub fn board_compact(&self) -> String {
        let mut result = String::new();
        for row in &self.board.cells {
            for &cell in row {
                result.push((b'0' + cell) as char);
            }
        }
        result
    }
}
