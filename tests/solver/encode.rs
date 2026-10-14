use wordle_solver::encode::{decode, try_encode};
use wordle_solver::types::WORD_LEN;

#[test]
fn roundtrips_real_words() {
    for w in [
        "soare", "emcee", "crypt", "molar", "aahed", "abaca", "abbey", "raise", "trace", "lares",
        "soare", "crane", "slate", "audio",
    ] {
        let code = try_encode(w).unwrap_or(0);
        assert_eq!(decode(code), w);
    }
}

#[test]
fn encoding_is_position_packed() {
    let w = try_encode("abcde").unwrap_or(0);
    for (i, c) in "abcde".bytes().enumerate() {
        assert_eq!((w >> (i * 8)) & 0xFF, u64::from(c - b'a'));
    }
}

#[test]
fn rejects_wrong_length() {
    assert!(try_encode("").is_none());
    assert!(try_encode("a").is_none());
    assert!(try_encode("abcd").is_none());
    assert!(try_encode("abcdef").is_none());
    assert!(try_encode("abcdefghijklmnopqrstuvwxyz").is_none());
}

#[test]
fn rejects_non_lowercase() {
    assert!(try_encode("ABCDE").is_none());
    assert!(try_encode("Soare").is_none());
    assert!(try_encode("soarE").is_none());
}

#[test]
fn rejects_non_letters() {
    assert!(try_encode("ab1de").is_none());
    assert!(try_encode("ab de").is_none());
    assert!(try_encode("ab.de").is_none());
    assert!(try_encode("{[{[{").is_none());
    assert!(try_encode("|}~|\x7f").is_none());
}

#[test]
fn rejects_multibyte_utf8() {
    assert!(try_encode("söare").is_none());
    assert!(try_encode("ab€de").is_none());
}

#[test]
fn accepts_boundaries() {
    assert_eq!(try_encode("aaaaa").unwrap_or(0), 0);
    assert_eq!(decode(0), "aaaaa");
    let z = try_encode("zzzzz").unwrap_or(0);
    assert_eq!(decode(z), "zzzzz");
}

#[test]
fn decode_of_an_encoded_word_always_yields_word_len() {
    for w in ["aaaaa", "zzzzz", "soare", "crane"] {
        let code = try_encode(w).unwrap_or(0);
        assert_eq!(decode(code).len(), WORD_LEN);
    }
}

#[test]
fn decode_never_panics_on_an_unencoded_value() {
    for code in [
        u64::MAX,
        0x0123_4567_89AB_CDEF,
        u64::from(u8::MAX) << 8,
        0xFF_FF,
    ] {
        let s = decode(code);
        assert_eq!(s.chars().count(), WORD_LEN, "code {code}");
    }
}

#[test]
fn decode_is_injective_on_encoded_words() {
    let mut seen = std::collections::HashSet::new();
    for w in ["aaaaa", "zzzzz", "soare", "crane", "abaca", "raise"] {
        let code = try_encode(w).unwrap_or(0);
        assert!(seen.insert(code), "{w} collides");
        assert_eq!(decode(code), w);
    }
}
