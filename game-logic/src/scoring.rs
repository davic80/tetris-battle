/// Scoring system following Tetris Guideline
pub struct Scoring {
    pub score: u32,
    pub level: u32,
    pub lines: u32,
    combo: i32,
    b2b: bool, // back-to-back (consecutive Tetris/T-spin)
    pending_attack: u32,
    gravity_counter: u32,
}

impl Scoring {
    pub fn new() -> Self {
        Scoring {
            score: 0,
            level: 1,
            lines: 0,
            combo: -1,
            b2b: false,
            pending_attack: 0,
            gravity_counter: 0,
        }
    }

    pub fn add_soft_drop(&mut self, rows: u32) {
        self.score += rows;
    }

    pub fn add_hard_drop(&mut self, rows: u32) {
        self.score += rows * 2;
    }

    /// Register a line clear, returns attack lines to send
    pub fn register_clear(&mut self, lines: u32) -> u32 {
        if lines == 0 {
            self.combo = -1;
            return 0;
        }
        self.combo += 1;
        self.lines += lines;
        self.level = (self.lines / 10) + 1;

        let is_difficult = lines == 4; // Tetris
        let b2b_bonus = if is_difficult && self.b2b { 1 } else { 0 };
        self.b2b = is_difficult;

        // Score points
        let base = match lines {
            1 => 100,
            2 => 300,
            3 => 500,
            4 => 800,
            _ => 800,
        };
        self.score += base * self.level + b2b_bonus * (base / 2) * self.level;

        // Combo bonus
        if self.combo > 0 {
            self.score += 50 * self.combo as u32 * self.level;
        }

        // Attack lines (garbage to send)
        let attack_base: u32 = match lines {
            1 => 0,
            2 => 1,
            3 => 2,
            4 => 4,
            _ => 0,
        };
        let combo_attack: u32 = match self.combo {
            0 => 0,
            1 => 1,
            2 => 1,
            3 => 2,
            4 => 2,
            5 => 3,
            6 => 3,
            _ => 4,
        };
        let total_attack = attack_base + b2b_bonus + combo_attack;
        self.pending_attack += total_attack;
        total_attack
    }

    pub fn take_attack(&mut self) -> u32 {
        let a = self.pending_attack;
        self.pending_attack = 0;
        a
    }

    /// Returns gravity interval in milliseconds based on level
    pub fn gravity_ms(&self) -> u32 {
        // Tetris Guideline gravity formula
        let level = self.level.min(20) as f64;
        let seconds = (0.8 - (level - 1.0) * 0.007).powf(level - 1.0);
        (seconds * 1000.0) as u32
    }

    /// Advance gravity timer. Returns true if piece should fall
    pub fn tick(&mut self) -> bool {
        false // Gravity is handled by JS timer
    }
}
