use crate::types::NUM_PATTERNS;

#[derive(Debug)]
pub struct ScoreTable {
    lut: Vec<f64>,
}

impl ScoreTable {
    #[must_use]
    pub fn entropy(max_count: usize) -> Self {
        let mut lut = Vec::with_capacity(max_count + 1);
        lut.push(0.0);
        for c in 1..=max_count {
            let f = c as f64;
            lut.push(f * f.log2());
        }
        Self { lut }
    }

    #[must_use]
    pub fn from_lut(lut: Vec<f64>) -> Self {
        Self { lut }
    }

    #[must_use]
    #[inline]
    pub fn bucket(&self, c: usize) -> f64 {
        self.lut[c]
    }
}

#[derive(Debug)]
pub struct Histogram {
    counts: [u16; NUM_PATTERNS],
    dirty: [u16; NUM_PATTERNS],
    dirty_len: u32,
    max_bucket: u16,
}

impl Histogram {
    #[must_use]
    pub fn new() -> Self {
        Self {
            counts: [0; NUM_PATTERNS],
            dirty: [0; NUM_PATTERNS],
            dirty_len: 0,
            max_bucket: 0,
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        let n = self.dirty_len as usize;
        for i in 0..n {
            self.counts[self.dirty[i] as usize] = 0;
        }
        self.dirty_len = 0;
        self.max_bucket = 0;
    }

    #[inline]
    pub fn add(&mut self, p: u8) {
        let c = &mut self.counts[p as usize];
        *c += 1;
        if *c == 1 {
            self.dirty[self.dirty_len as usize] = u16::from(p);
            self.dirty_len += 1;
        }
        if *c > self.max_bucket {
            self.max_bucket = *c;
        }
    }

    #[must_use]
    #[inline]
    pub fn score(&self, table: &ScoreTable) -> f64 {
        let mut s = 0.0f64;
        for i in 0..self.dirty_len as usize {
            s += table.bucket(usize::from(self.counts[self.dirty[i] as usize]));
        }
        s
    }

    #[must_use]
    pub fn nonzero(&self) -> u16 {
        u16::try_from(self.dirty_len).unwrap_or(u16::MAX)
    }

    #[must_use]
    pub fn max_bucket(&self) -> u16 {
        self.max_bucket
    }
}

impl Default for Histogram {
    fn default() -> Self {
        Self::new()
    }
}
