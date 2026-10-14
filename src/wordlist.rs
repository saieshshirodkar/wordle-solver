use crate::encode::{decode, try_encode};
use crate::error::{Error, Result};
use crate::types::{WORD_LEN, Word};

#[inline]
fn rank(w: Word) -> u32 {
    let mut n: u32 = 0;
    let mut m: u32 = 1;
    for i in 0..WORD_LEN {
        let l = (w >> (i * 8)) & 0xFF;
        n += l as u32 * m;
        m *= 26;
    }
    n
}

pub fn load_words(path: &str) -> Result<Vec<Word>> {
    let text = std::fs::read_to_string(path)?;

    let mut out: Vec<Word> = Vec::with_capacity(1 << 13);
    let mut ranks: Vec<u32> = Vec::with_capacity(1 << 13);

    for raw in text.split_ascii_whitespace() {
        if raw.len() != WORD_LEN {
            return Err(Error::BadLength {
                path: path.to_owned(),
                word: raw.to_owned(),
            });
        }

        let Some(w) = try_encode(raw) else {
            return Err(Error::BadChar {
                path: path.to_owned(),
                word: raw.to_owned(),
            });
        };

        ranks.push(rank(w));
        out.push(w);
    }

    if out.is_empty() {
        return Err(Error::Empty {
            path: path.to_owned(),
        });
    }

    ranks.sort_unstable();
    if let Some(dup) = ranks.windows(2).find(|p| p[0] == p[1]).map(|p| p[0]) {
        let word = out
            .iter()
            .copied()
            .find(|&w| rank(w) == dup)
            .unwrap_or_default();
        return Err(Error::Duplicate {
            path: path.to_owned(),
            word: decode(word),
        });
    }

    Ok(out)
}
