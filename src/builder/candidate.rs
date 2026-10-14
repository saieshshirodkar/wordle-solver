const EPS: f64 = 1e-12;

pub const NO_INDEX: u16 = u16::MAX;

#[must_use]
pub fn threads_available() -> usize {
    std::thread::available_parallelism().map_or(1, std::num::NonZero::get)
}

#[derive(Clone, Copy, Debug)]
pub struct Candidate {
    pub selection: f64,
    pub max_bucket: u16,
    pub nonzero: u16,
    pub index: u16,
    pub is_answer: bool,
}

impl Default for Candidate {
    fn default() -> Self {
        Self {
            selection: f64::INFINITY,
            max_bucket: 0,
            nonzero: 0,
            index: NO_INDEX,
            is_answer: false,
        }
    }
}

impl Candidate {
    #[must_use]
    #[inline]
    pub fn perfect(&self) -> bool {
        self.index != NO_INDEX && self.is_answer && self.max_bucket == 1
    }
}

#[must_use]
#[inline]
pub fn candidate_better(a: &Candidate, b: &Candidate) -> bool {
    if a.index == NO_INDEX {
        return false;
    }
    if b.index == NO_INDEX {
        return true;
    }
    let (pa, pb) = (a.perfect(), b.perfect());
    if pa != pb {
        return pa;
    }
    if pa {
        return a.index < b.index;
    }
    if a.selection < b.selection - EPS {
        return true;
    }
    if a.selection > b.selection + EPS {
        return false;
    }
    if a.is_answer != b.is_answer {
        return a.is_answer;
    }
    if a.max_bucket != b.max_bucket {
        return a.max_bucket < b.max_bucket;
    }
    if a.nonzero != b.nonzero {
        return a.nonzero > b.nonzero;
    }
    a.index < b.index
}
