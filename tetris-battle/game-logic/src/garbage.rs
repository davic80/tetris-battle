// garbage.rs - re-exported utilities (main logic is in Board::add_garbage)
// This module is reserved for future garbage configuration (messiness, etc.)

pub fn calc_attack(lines_cleared: u32, b2b: bool, combo: i32) -> u32 {
    let base: u32 = match lines_cleared {
        1 => 0,
        2 => 1,
        3 => 2,
        4 => 4,
        _ => 0,
    };
    let b2b_bonus = if b2b && lines_cleared == 4 { 1 } else { 0 };
    let combo_bonus: u32 = match combo {
        0 => 0,
        1 | 2 => 1,
        3 | 4 => 2,
        5 | 6 => 3,
        _ => 4,
    };
    base + b2b_bonus + combo_bonus
}
