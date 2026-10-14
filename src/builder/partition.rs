use crate::pattern::pattern;
use crate::types::ALL_GREEN;

use super::{Builder, MAX_HIST};

pub(super) type Split = [(u8, u32, u32); MAX_HIST];

impl Builder {
    pub(super) fn partition(
        &mut self,
        level: usize,
        start: usize,
        len: usize,
        guess: u16,
    ) -> (Split, usize) {
        let gw = self.guesses[usize::from(guess)];

        let mut counts = [0u32; MAX_HIST];
        for k in 0..len {
            let i = usize::from(self.levels[level][start + k]);
            counts[usize::from(pattern(gw, self.answers[i], &self.counts[i]))] += 1;
        }

        let mut offs = [0u32; MAX_HIST];
        let mut pos = 0u32;
        for (p, &c) in counts.iter().enumerate() {
            offs[p] = pos;
            if p != ALL_GREEN as usize {
                pos += c;
            }
        }

        let next = level + 1;
        self.levels[next].clear();
        self.levels[next].resize(pos as usize, 0);

        let mut write = offs;
        for k in 0..len {
            let ai = self.levels[level][start + k];
            let i = usize::from(ai);
            let p = usize::from(pattern(gw, self.answers[i], &self.counts[i]));
            if p != ALL_GREEN as usize {
                self.levels[next][write[p] as usize] = ai;
                write[p] += 1;
            }
        }

        let mut live = [(0u8, 0u32, 0u32); MAX_HIST];
        let mut nlive = 0usize;
        for (p, &c) in counts.iter().enumerate() {
            if p != ALL_GREEN as usize && c != 0 {
                live[nlive] = (p as u8, c, offs[p]);
                nlive += 1;
            }
        }
        (live, nlive)
    }

    pub(super) fn mark_answers(&mut self, level: usize, start: usize, len: usize, on: bool) {
        let base = self.num_probes;
        for &ai in &self.levels[level][start..start + len] {
            self.is_answer[base + usize::from(ai)] = on;
        }
    }
}
