pub type Word = u64;
pub type Pattern = u8;

pub const WORD_LEN: usize = 5;
pub const NUM_PATTERNS: usize = 243;
pub const NUM_LETTERS: usize = 26;
pub const ALL_GREEN: Pattern = 242;
pub const MAX_ANSWERS: usize = 65_535;
pub const MAX_GUESSES: i32 = 6;
pub const POW3: [u16; WORD_LEN] = [1, 3, 9, 27, 81];

#[must_use]
#[inline]
pub const fn letter_at(w: Word, i: usize) -> u8 {
    ((w >> (i * 8)) & 0xFF) as u8
}
