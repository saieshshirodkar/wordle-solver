use crate::types::{WORD_LEN, Word};

#[must_use]
#[inline]
pub fn try_encode(s: &str) -> Option<Word> {
    let b = s.as_bytes();
    if b.len() != WORD_LEN {
        return None;
    }
    let mut w: Word = 0;
    for (i, &c) in b.iter().enumerate() {
        let l = c.checked_sub(b'a')?;
        if l > 25 {
            return None;
        }
        w |= Word::from(l) << (i * 8);
    }
    Some(w)
}

#[must_use]
pub fn decode(w: Word) -> String {
    let mut s = String::with_capacity(WORD_LEN);
    for i in 0..WORD_LEN {
        let l = (w >> (i * 8)) & 0xFF;
        let l = u8::try_from(l).unwrap_or(0);
        s.push((b'a'.saturating_add(l)) as char);
    }
    s
}
