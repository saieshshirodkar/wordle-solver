use crate::types::{ALL_GREEN, NUM_LETTERS, NUM_PATTERNS, POW3, WORD_LEN, Word};

pub type Counts = [u8; NUM_LETTERS];

#[must_use]
#[inline]
pub fn letter_counts(w: Word) -> Counts {
    let mut c = [0u8; NUM_LETTERS];
    for i in 0..WORD_LEN {
        c[((w >> (i * 8)) & 0xFF) as usize] += 1;
    }
    c
}

#[must_use]
#[inline]
pub fn pattern(guess: Word, target: Word, pre: &Counts) -> u8 {
    let mut cnt = *pre;
    let mut pat: u16 = 0;

    let mut g = guess;
    let mut t = target;
    for p in POW3 {
        let gc = (g & 0xFF) as usize;
        if gc == (t & 0xFF) as usize {
            pat += 2 * p;
            cnt[gc] -= 1;
        }
        g >>= 8;
        t >>= 8;
    }

    g = guess;
    t = target;
    for p in POW3 {
        let gc = (g & 0xFF) as usize;
        if gc != (t & 0xFF) as usize && cnt[gc] != 0 {
            pat += p;
            cnt[gc] -= 1;
        }
        g >>= 8;
        t >>= 8;
    }

    pat as u8
}

#[must_use]
#[inline]
pub fn pattern_reference(guess: Word, target: Word) -> u8 {
    let mut cnt = letter_counts(target);
    let mut pat: u16 = 0;
    for (i, p) in POW3.iter().enumerate() {
        let gc = ((guess >> (i * 8)) & 0xFF) as usize;
        if gc == ((target >> (i * 8)) & 0xFF) as usize {
            pat += 2 * p;
            cnt[gc] -= 1;
        }
    }
    for (i, p) in POW3.iter().enumerate() {
        let gc = ((guess >> (i * 8)) & 0xFF) as usize;
        if gc != ((target >> (i * 8)) & 0xFF) as usize && cnt[gc] != 0 {
            pat += p;
            cnt[gc] -= 1;
        }
    }
    pat as u8
}

#[must_use]
pub fn parse_feedback(s: &str) -> Option<u8> {
    if s.len() != WORD_LEN {
        return None;
    }
    let b = s.as_bytes();
    let mut pat: u16 = 0;
    for (i, p) in POW3.iter().enumerate() {
        let d: u16 = match b[i] {
            b'g' | b'G' => 2,
            b'y' | b'Y' => 1,
            b'b' | b'B' => 0,
            _ => return None,
        };
        pat += d * p;
    }
    Some(pat as u8)
}

const _: () = assert!(NUM_PATTERNS == 243);
const _: () = assert!(POW3[4] * 3 == 243);
const _: () = assert!(ALL_GREEN == (POW3[0] + POW3[1] + POW3[2] + POW3[3] + POW3[4]) as u8 * 2);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::{decode, try_encode};

    fn enc(s: &str) -> Word {
        try_encode(s).unwrap_or(0)
    }

    fn small_alphabet(n: u64) -> Vec<Word> {
        let mut v = Vec::with_capacity((n * n * n * n * n) as usize);
        for a in 0..n {
            for b in 0..n {
                for c in 0..n {
                    for d in 0..n {
                        for e in 0..n {
                            v.push(a | (b << 8) | (c << 16) | (d << 24) | (e << 32));
                        }
                    }
                }
            }
        }
        v
    }

    #[test]
    fn known_patterns() {
        let cases = [
            ("abcde", "abcde", ALL_GREEN),
            ("abcde", "fghij", 0),
            ("abcde", "fghci", 9),
            ("abbey", "about", 8),
            ("abbey", "abode", 35),
        ];
        for (g, t, want) in cases {
            let (gw, tw) = (enc(g), enc(t));
            assert_eq!(pattern(gw, tw, &letter_counts(tw)), want, "{g}/{t}");
            assert_eq!(pattern_reference(gw, tw), want, "ref {g}/{t}");
        }
    }

    #[test]
    fn fast_matches_reference_exhaustive() {
        let ws = small_alphabet(5);
        let pre: Vec<Counts> = ws.iter().copied().map(letter_counts).collect();
        let mut checked: u64 = 0;
        for (ti, &t) in ws.iter().enumerate() {
            for &g in &ws {
                assert_eq!(
                    pattern(g, t, &pre[ti]),
                    pattern_reference(g, t),
                    "{} / {}",
                    decode(g),
                    decode(t)
                );
                checked += 1;
            }
        }
        assert_eq!(checked, 9_765_625);
    }

    #[test]
    fn feedback_parsing() {
        assert_eq!(parse_feedback("ggggg"), Some(ALL_GREEN));
        assert_eq!(parse_feedback("bbbbb"), Some(0));
        assert_eq!(parse_feedback("bgybb"), Some(15));
        assert_eq!(parse_feedback("GGGGG"), Some(ALL_GREEN));
        assert_eq!(parse_feedback("bgyb"), None);
        assert_eq!(parse_feedback("bgybx"), None);
        assert_eq!(parse_feedback("bgybz"), None);
        assert_eq!(parse_feedback(""), None);
    }

    #[test]
    fn counts_multiset() {
        let c = letter_counts(enc("aabbb"));
        assert_eq!(c[0], 2);
        assert_eq!(c[1], 3);
        assert_eq!(c.iter().map(|&x| u32::from(x)).sum::<u32>(), 5);
    }
}
