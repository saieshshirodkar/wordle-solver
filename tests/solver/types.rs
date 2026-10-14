use wordle_solver::types::{
    ALL_GREEN, MAX_ANSWERS, MAX_GUESSES, NUM_LETTERS, NUM_PATTERNS, POW3, WORD_LEN, letter_at,
};

#[test]
fn constants_are_self_consistent() {
    assert_eq!(WORD_LEN, 5);
    assert_eq!(NUM_PATTERNS, 3 * POW3[WORD_LEN - 1] as usize);
    assert_eq!(NUM_LETTERS, 26);
    assert_eq!(
        ALL_GREEN as usize,
        2 * POW3.iter().map(|&p| p as usize).sum::<usize>()
    );
    assert_eq!(MAX_ANSWERS, 65_535);
    assert_eq!(MAX_GUESSES, 6);
}

#[test]
fn pow3_weights_are_positional() {
    assert_eq!(POW3, [1, 3, 9, 27, 81]);
    for i in 1..WORD_LEN {
        assert_eq!(POW3[i], POW3[i - 1] * 3);
    }
}

#[test]
fn letter_at_reads_every_position() {
    let w = (0u64..5).fold(0u64, |acc, i| acc | (i << (i * 8)));
    for i in 0..WORD_LEN {
        assert_eq!(letter_at(w, i), u8::try_from(i).unwrap_or(0));
    }
}

#[test]
fn letter_at_ignores_neighbouring_bytes() {
    let w = (0u64..5).fold(0u64, |acc, i| acc | (0xFFu64 << (i * 8)));
    for i in 0..WORD_LEN {
        assert_eq!(letter_at(w, i), 0xFF);
    }
}
