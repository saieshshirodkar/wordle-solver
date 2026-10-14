use crate::error::{Error, Result};
use crate::pattern::{Counts, letter_counts, pattern};
use crate::tree::{NONE, Tree};
use crate::types::{ALL_GREEN, Word};

#[derive(Debug)]
pub struct Secrets<'a> {
    words: &'a [Word],
    counts: &'a [Counts],
}

impl Secrets<'_> {
    #[must_use]
    #[inline]
    fn word(&self, i: usize) -> Word {
        self.words[i]
    }

    #[must_use]
    #[inline]
    fn counts(&self, i: usize) -> &Counts {
        &self.counts[i]
    }
}

impl<'a> Secrets<'a> {
    #[must_use]
    pub fn new(words: &'a [Word], counts: &'a [Counts]) -> Self {
        debug_assert_eq!(words.len(), counts.len());
        Self { words, counts }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.words.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }
}

#[must_use]
pub fn secrets(words: &[Word]) -> Vec<Counts> {
    words.iter().copied().map(letter_counts).collect()
}

#[must_use]
#[inline]
pub fn walk(tree: &Tree, guesses: &[Word], s: &Secrets<'_>, secret: usize) -> i32 {
    let mut node: u16 = 0;
    let mut made: i32 = 0;
    loop {
        made += 1;
        if tree.terminal(node) {
            return made;
        }
        let fb = pattern(
            guesses[tree.guess_at(node) as usize],
            s.word(secret),
            s.counts(secret),
        );
        if fb == ALL_GREEN {
            return made;
        }
        let next = tree.lookup(node, fb);
        if next == NONE {
            return -1;
        }
        node = next;
    }
}

pub fn solve_one(tree: &Tree, guesses: &[Word], s: &Secrets<'_>, secret: i32) -> Result<i32> {
    let Ok(idx) = usize::try_from(secret) else {
        return Err(Error::BadSecret(secret));
    };
    if idx >= s.len() {
        return Err(Error::BadSecret(secret));
    }
    Ok(walk(tree, guesses, s, idx))
}

#[derive(Debug, Default)]
pub struct Bench {
    pub counts: Vec<i32>,
    pub total: i64,
    pub avg: f64,
    pub stddev: f64,
    pub worst: i32,
    pub failed: usize,
}

#[must_use]
pub fn run_bench(tree: &Tree, guesses: &[Word], s: &Secrets<'_>, threads: usize) -> Bench {
    let n = s.len();
    let lanes = threads.max(1);
    let chunk = n.div_ceil(lanes);
    let mut counts = vec![-1i32; n];

    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..lanes)
            .map(|lane| {
                let start = (lane * chunk).min(n);
                let stop = (start + chunk).min(n);
                scope.spawn(move || {
                    let mut local = vec![0i32; stop - start];
                    for (offset, slot) in local.iter_mut().enumerate() {
                        *slot = walk(tree, guesses, s, start + offset);
                    }
                    local
                })
            })
            .collect();

        for (lane, handle) in handles.into_iter().enumerate() {
            if let Ok(local) = handle.join() {
                let start = (lane * chunk).min(n);
                counts[start..start + local.len()].copy_from_slice(&local);
            }
        }
    });

    let mut total: i64 = 0;
    let mut worst = 0;
    let mut solved = 0usize;
    for &c in &counts {
        if c <= 0 {
            continue;
        }
        total += i64::from(c);
        worst = worst.max(c);
        solved += 1;
    }

    let avg = if solved == 0 {
        0.0
    } else {
        total as f64 / solved as f64
    };
    let mut sq = 0.0f64;
    for &c in &counts {
        if c <= 0 {
            continue;
        }
        let d = f64::from(c) - avg;
        sq += d * d;
    }

    Bench {
        failed: n - solved,
        counts,
        total,
        avg,
        stddev: if solved == 0 {
            0.0
        } else {
            (sq / solved as f64).sqrt()
        },
        worst,
    }
}
