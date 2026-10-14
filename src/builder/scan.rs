use crate::error::{Error, Result};
use crate::pattern::pattern;
use crate::scoring::Histogram;

use super::Builder;
use super::NO_INDEX;
use super::candidate::{Candidate, candidate_better, threads_available};

impl Builder {
    pub(super) fn scan(&self, level: usize, lo: u16, hi: u16, threads: usize) -> Candidate {
        let off = self.offsets[level];
        let len = self.lengths[level];
        let rem = &self.levels[level][off..off + len];

        let span = usize::from(hi - lo);
        let chunk = span.div_ceil(threads) as u16;

        let run = |a: u16, b: u16| -> Candidate {
            let mut hist = Histogram::new();
            let mut best = Candidate::default();

            for gi in a..b {
                let g = self.guesses[usize::from(gi)];
                hist.reset();

                for &ai in rem {
                    let i = usize::from(ai);
                    hist.add(pattern(g, self.answers[i], &self.counts[i]));
                }

                let cand = Candidate {
                    selection: hist.score(&self.scores),
                    max_bucket: hist.max_bucket(),
                    nonzero: hist.nonzero(),
                    index: gi,
                    is_answer: self.is_answer[usize::from(gi)],
                };
                if candidate_better(&cand, &best) {
                    best = cand;
                }
            }
            best
        };

        if threads == 1 || chunk == 0 {
            return run(lo, hi);
        }

        let mut best = Candidate::default();
        std::thread::scope(|s| {
            let handles: Vec<_> = (0..threads)
                .map(|t| {
                    let shift = usize::from(chunk) * t;
                    let a = lo
                        .saturating_add(u16::try_from(shift).unwrap_or(u16::MAX))
                        .min(hi);
                    let b = a.saturating_add(chunk).min(hi);
                    s.spawn(move || run(a, b))
                })
                .collect();
            for handle in handles {
                if let Ok(c) = handle.join() {
                    if candidate_better(&c, &best) {
                        best = c;
                    }
                }
            }
        });

        best
    }

    pub(super) fn threads_for(&self, work: usize) -> usize {
        let hw = match std::env::var("WORDLE_THREADS") {
            Ok(v) => match v.parse::<usize>() {
                Ok(n) => n.max(1),
                Err(_) => threads_available(),
            },
            Err(_) => threads_available(),
        };
        let per = self.answers.len().max(1) * 40;
        hw.min((work / per).max(1))
    }

    pub(super) fn pick(&self, level: usize, node: u16, threads: usize) -> Result<u16> {
        let hi = u16::try_from(self.guesses.len()).unwrap_or(u16::MAX);
        let best = self.scan(level, 0, hi, threads);
        if best.index == NO_INDEX {
            return Err(Error::NoCandidate { node });
        }
        Ok(best.index)
    }
}
