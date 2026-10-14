use wordle_solver::error::Error;
use wordle_solver::types::Word;
use wordle_solver::wordlist::load_words;

use super::{answers, probes, temp};

#[test]
fn loads_the_curated_lists() {
    let a = answers();
    let p = probes();
    assert_eq!(a.len(), 2315, "official Wordle answer list");
    assert_eq!(p.len(), 10657, "official non-answer guess list");
}

#[test]
fn the_two_lists_are_disjoint_and_total_12972() {
    let a = answers();
    let p = probes();
    assert_eq!(
        a.len() + p.len(),
        12_972,
        "the official allowed-guess count is 12972"
    );
    for w in &p {
        assert!(!a.contains(w), "probe list overlaps the answer list");
    }
}

#[test]
fn the_lists_are_ordered_and_unique() {
    for list in [answers(), probes()] {
        let words: Vec<String> = list
            .iter()
            .copied()
            .map(wordle_solver::encode::decode)
            .collect();
        let mut sorted = words.clone();
        sorted.sort();
        assert_eq!(words, sorted, "list is not in sorted order");

        let mut dedup = words.clone();
        dedup.dedup();
        assert_eq!(words, dedup, "list contains duplicates");
    }
}

#[test]
fn every_word_is_five_lowercase_letters() {
    for list in [answers(), probes()] {
        for &w in &list {
            let s = wordle_solver::encode::decode(w);
            assert_eq!(s.len(), 5, "{s}");
            assert!(s.bytes().all(|b| b.is_ascii_lowercase()), "{s}");
        }
    }
}

#[test]
fn files_end_with_a_newline() {
    for path in ["data/answers.txt", "data/guesses.txt"] {
        let raw = std::fs::read(path).unwrap_or_default();
        assert_eq!(raw.last(), Some(&b'\n'), "{path} has no trailing newline");
    }
}

#[test]
fn a_missing_file_is_an_error() {
    assert!(load_words("definitely-not-here.txt").is_err());
}

#[test]
fn an_empty_file_is_an_error() {
    let p = temp("wordle-rs-wl-empty.txt");
    std::fs::write(&p, "").unwrap_or_default();
    assert!(load_words(&p).is_err());
    let _ = std::fs::remove_file(&p);
}

#[test]
fn whitespace_only_is_an_error() {
    let p = temp("wordle-rs-wl-blank.txt");
    std::fs::write(&p, "\n\n   \n\t\n").unwrap_or_default();
    assert!(load_words(&p).is_err());
    let _ = std::fs::remove_file(&p);
}

#[test]
fn a_word_of_the_wrong_length_is_rejected() {
    for body in ["soar\n", "soaree\n", "s\n"] {
        let p = temp("wordle-rs-wl-len.txt");
        std::fs::write(&p, body).unwrap_or_default();
        let e = load_words(&p).unwrap_err();
        assert!(matches!(e, Error::BadLength { .. }), "{body:?} gave {e}");
        let _ = std::fs::remove_file(&p);
    }
}

#[test]
fn a_non_a_z_character_is_rejected() {
    for body in ["so4re\n", "so.re\n", "s_rex\n", "s-rex\n", "sore\\"] {
        let p = temp("wordle-rs-wl-char.txt");
        std::fs::write(&p, body).unwrap_or_default();
        let e = load_words(&p).unwrap_err();
        assert!(matches!(e, Error::BadChar { .. }), "{body:?} gave {e}");
        let _ = std::fs::remove_file(&p);
    }
}

#[test]
fn a_multibyte_word_is_a_length_error_first() {
    let p = temp("wordle-rs-wl-mb.txt");
    std::fs::write(&p, "söare\n").unwrap_or_default();
    let e = load_words(&p).unwrap_err();
    assert!(
        matches!(e, Error::BadLength { .. }),
        "a 6-byte token should fail on length, got {e}"
    );
    let _ = std::fs::remove_file(&p);
}

#[test]
fn uppercase_is_rejected() {
    let p = temp("wordle-rs-wl-upper.txt");
    std::fs::write(&p, "SOARE\n").unwrap_or_default();
    let e = load_words(&p).unwrap_err();
    assert!(matches!(e, Error::BadChar { .. }), "{e}");
    let _ = std::fs::remove_file(&p);
}

#[test]
fn a_duplicate_is_rejected() {
    let p = temp("wordle-rs-wl-dup.txt");
    std::fs::write(&p, "soare\ncrane\nsoare\n").unwrap_or_default();
    let e = load_words(&p).unwrap_err();
    match e {
        Error::Duplicate { word, .. } => assert_eq!(word, "soare"),
        other => panic!("expected a duplicate error, got {other}"),
    }
    let _ = std::fs::remove_file(&p);
}

#[test]
fn a_duplicate_is_caught_regardless_of_position() {
    for body in ["soare\nsoare\n", "soare\ncrane\nslate\nsoare\n"] {
        let p = temp("wordle-rs-wl-dup2.txt");
        std::fs::write(&p, body).unwrap_or_default();
        assert!(load_words(&p).is_err(), "{body:?}");
        let _ = std::fs::remove_file(&p);
    }
}

#[test]
fn words_are_split_on_any_whitespace() {
    let p = temp("wordle-rs-wl-ws.txt");
    std::fs::write(&p, "soare crane\tslate\n\r\nraise\n").unwrap_or_default();
    let loaded = load_words(&p).unwrap_or_default();
    let _ = std::fs::remove_file(&p);

    let want: Vec<Word> = ["soare", "crane", "slate", "raise"]
        .iter()
        .map(|w| wordle_solver::encode::try_encode(w).unwrap_or(0))
        .collect();
    assert_eq!(loaded, want);
}

#[test]
fn order_is_preserved_from_the_file() {
    let p = temp("wordle-rs-wl-order.txt");
    std::fs::write(&p, "zzzxx\naaaaa\nmmmmm\n").unwrap_or_default();
    let loaded = load_words(&p).unwrap_or_default();
    let _ = std::fs::remove_file(&p);

    let want: Vec<Word> = ["zzzxx", "aaaaa", "mmmmm"]
        .iter()
        .map(|w| wordle_solver::encode::try_encode(w).unwrap_or(0))
        .collect();
    assert_eq!(loaded, want, "load_words must not sort its output");
}

#[test]
fn a_single_word_file_loads() {
    let p = temp("wordle-rs-wl-one.txt");
    std::fs::write(&p, "soare\n").unwrap_or_default();
    assert_eq!(load_words(&p).unwrap_or_default().len(), 1);
    let _ = std::fs::remove_file(&p);
}
