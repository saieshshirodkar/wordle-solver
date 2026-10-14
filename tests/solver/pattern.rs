use wordle_solver::encode::try_encode;
use wordle_solver::pattern::{Counts, letter_counts, parse_feedback, pattern, pattern_reference};
use wordle_solver::types::{ALL_GREEN, NUM_LETTERS, POW3, WORD_LEN, Word};

use super::greedy;

fn enc(s: &str) -> Word {
    try_encode(s).unwrap_or(0)
}

fn counts(s: &str) -> Counts {
    letter_counts(enc(s))
}

#[test]
fn letter_counts_tallies_repeats() {
    let c = counts("aabbb");
    assert_eq!(c[0], 2);
    assert_eq!(c[1], 3);
    assert_eq!(c[2], 0);
    assert_eq!(
        c.iter().map(|&x| u32::from(x)).sum::<u32>(),
        u32::try_from(WORD_LEN).unwrap_or(0)
    );
}

#[test]
fn letter_counts_are_position_independent() {
    for w in ["soare", "abbey", "crane", "aahed"] {
        let c = counts(w);
        assert_eq!(
            c.iter().map(|&x| u32::from(x)).sum::<u32>(),
            u32::try_from(WORD_LEN).unwrap_or(0)
        );
        assert_eq!(c.len(), NUM_LETTERS);
    }
}

#[test]
fn identical_words_are_all_green() {
    for w in ["abcde", "soare", "zzzzz", "aahed"] {
        assert_eq!(pattern(enc(w), enc(w), &counts(w)), ALL_GREEN, "{w}");
        assert_eq!(pattern_reference(enc(w), enc(w)), ALL_GREEN, "ref {w}");
    }
}

#[test]
fn disjoint_words_are_all_gray() {
    assert_eq!(pattern(enc("abcde"), enc("fghij"), &counts("fghij")), 0);
    assert_eq!(pattern_reference(enc("abcde"), enc("fghij")), 0);
}

#[test]
fn known_patterns_match_the_reference() {
    let cases = [
        ("abcde", "abcde", ALL_GREEN),
        ("abcde", "fghij", 0),
        ("abcde", "fghci", 9),
        ("abbey", "about", 8),
        ("abbey", "abode", 35),
    ];
    for (g, t, want) in cases {
        assert_eq!(pattern(enc(g), enc(t), &counts(t)), want, "{g}/{t}");
        assert_eq!(pattern_reference(enc(g), enc(t)), want, "ref {g}/{t}");
    }
}

#[test]
fn fast_path_agrees_with_the_reference_on_the_real_dictionary() {
    let (_, _, answers) = greedy();
    let g = enc("soare");
    for &t in answers.iter().take(400) {
        assert_eq!(
            pattern(g, t, &letter_counts(t)),
            pattern_reference(g, t),
            "{} / {}",
            wordle_solver::encode::decode(g),
            wordle_solver::encode::decode(t)
        );
    }
}

#[test]
fn pattern_is_symmetric_in_greens() {
    let a = enc("soare");
    let b = enc("crane");
    let pa = pattern(a, b, &counts("crane"));
    let pb = pattern(b, a, &counts("soare"));
    let green_a = count_digit(pa, 2);
    let green_b = count_digit(pb, 2);
    assert!(green_a >= green_b);
    assert_eq!(
        green_a,
        (0..WORD_LEN)
            .filter(|&i| letter_at(a, i) == letter_at(b, i))
            .count()
    );
}

fn count_digit(p: u8, digit: u8) -> usize {
    let p = usize::from(p);
    POW3.iter()
        .take(WORD_LEN)
        .filter(|&&w| (p / usize::from(w)) % 3 == usize::from(digit))
        .count()
}

fn letter_at(w: Word, i: usize) -> u8 {
    u8::try_from((w >> (i * 8)) & 0xFF).unwrap_or(0)
}

#[test]
fn repeated_letter_marks_only_what_it_must() {
    let g = enc("eexxx");
    let t = enc("aaeex");
    let p = pattern(g, t, &counts("aaeex"));
    let ref_p = pattern_reference(g, t);
    assert_eq!(p, ref_p);
    assert!(count_digit(p, 2) <= 2);
}

#[test]
fn pattern_always_fits_a_digit_per_position() {
    let g = enc("abbey");
    let t = enc("about");
    let p = usize::from(pattern(g, t, &counts("about")));
    for &w in POW3.iter().take(WORD_LEN) {
        assert!((p / usize::from(w)) % 3 <= 2);
    }
}

#[test]
fn feedback_parsing_accepts_both_cases() {
    assert_eq!(parse_feedback("ggggg"), Some(ALL_GREEN));
    assert_eq!(parse_feedback("GGGGG"), Some(ALL_GREEN));
    assert_eq!(parse_feedback("bbbbb"), Some(0));
    assert_eq!(parse_feedback("BBBBB"), Some(0));
    assert_eq!(parse_feedback("bgybb"), Some(15));
    assert_eq!(parse_feedback("BgYbB"), Some(15));
}

#[test]
fn feedback_parsing_rejects_bad_input() {
    assert_eq!(parse_feedback(""), None);
    assert_eq!(parse_feedback("bgyb"), None);
    assert_eq!(parse_feedback("bgybx"), None);
    assert_eq!(parse_feedback("bgybz"), None);
    assert_eq!(parse_feedback("gggggg"), None);
    assert_eq!(parse_feedback("xxxxx"), None);
    assert_eq!(parse_feedback("bgyb1"), None);
    assert_eq!(parse_feedback("bgyé"), None);
}

#[test]
fn parsed_feedback_matches_a_computed_pattern() {
    for (g, t, s) in [
        ("soare", "crane", "bbgyg"),
        ("soare", "somen", "ggbby"),
        ("abbey", "about", "ggbbb"),
    ] {
        let p = parse_feedback(s).unwrap_or(255);
        assert_eq!(p, pattern(enc(g), enc(t), &counts(t)), "{g}/{t} {s}");
    }
}

#[test]
fn every_feedback_string_round_trips_through_its_digits() {
    for s in ["ggggg", "bbbbb", "gygyg", "ybbyb", "gbygb", "bygyg"] {
        let p = usize::from(parse_feedback(s).unwrap_or(255));
        let mut rebuilt = String::with_capacity(WORD_LEN);
        for &w in POW3.iter().take(WORD_LEN) {
            let d = (p / usize::from(w)) % 3;
            rebuilt.push(['b', 'y', 'g'][d]);
        }
        assert_eq!(
            usize::from(parse_feedback(&rebuilt).unwrap_or(0)),
            p,
            "{s} -> {rebuilt}"
        );
        assert_eq!(rebuilt.len(), WORD_LEN);
    }
}

#[test]
fn every_digit_position_is_independent() {
    for (i, &w) in POW3.iter().enumerate() {
        for d in ['b', 'y', 'g'] {
            let mut s = ['b'; WORD_LEN];
            s[i] = d;
            let text: String = s.iter().collect();
            let p = parse_feedback(&text).unwrap_or(255);
            let want = match d {
                'b' => 0u8,
                'y' => u8::try_from(w).unwrap_or(0),
                _ => u8::try_from(2 * u32::from(w)).unwrap_or(0),
            };
            assert_eq!(p, want, "position {i} digit {d}");
        }
    }
}
