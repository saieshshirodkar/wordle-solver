use crate::types::{NUM_PATTERNS, Pattern};

#[derive(Default, Clone)]
pub struct Tree {
    pub guess: Vec<u16>,
    pub offset: Vec<u32>,
    pub count: Vec<u8>,
    pub pattern: Vec<u8>,
    pub child: Vec<u16>,
    pub weight: Vec<u32>,
}

pub const NONE: u16 = 0xFFFF;

impl std::fmt::Debug for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tree")
            .field("nodes", &self.guess.len())
            .field("edges", &self.pattern.len())
            .finish_non_exhaustive()
    }
}

impl Tree {
    pub fn push_node(&mut self, offset: u32, count: u8, guess: u16, weight: u32) -> u16 {
        let id = self.guess.len() as u16;
        self.guess.push(guess);
        self.offset.push(offset);
        self.count.push(count);
        self.weight.push(weight);
        id
    }

    #[must_use]
    #[inline]
    pub fn lookup(&self, node: u16, p: Pattern) -> u16 {
        let off = self.offset[node as usize] as usize;
        let cnt = self.count[node as usize] as usize;
        let pats = &self.pattern[off..off + cnt];
        match pats.iter().position(|&x| x == p) {
            Some(i) => self.child[off + i],
            None => NONE,
        }
    }

    #[must_use]
    #[inline]
    pub fn guess_at(&self, n: u16) -> u16 {
        self.guess[n as usize]
    }

    #[must_use]
    #[inline]
    pub fn terminal(&self, n: u16) -> bool {
        self.count[n as usize] == 0
    }

    #[must_use]
    pub fn node_count(&self) -> usize {
        self.guess.len()
    }

    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.pattern.len()
    }

    #[must_use]
    pub fn costs(&self) -> Vec<f64> {
        let n = self.node_count();
        let mut cost = vec![0.0f64; n];
        for i in (0..n).rev() {
            if self.count[i] == 0 {
                cost[i] = 1.0;
                continue;
            }
            let total = f64::from(self.weight[i]);
            let off = self.offset[i] as usize;
            let cnt = self.count[i] as usize;
            let mut acc = 0.0f64;
            for e in off..off + cnt {
                let child = self.child[e] as usize;
                acc += f64::from(self.weight[child]) / total * cost[child];
            }
            cost[i] = 1.0 + acc;
        }
        cost
    }

    #[must_use]
    pub fn root_cost(&self) -> f64 {
        self.costs().first().copied().unwrap_or(0.0)
    }

    #[must_use]
    pub fn remaining_at(&self, n: u16) -> u32 {
        self.weight[n as usize]
    }

    #[must_use]
    pub fn validate(&self) -> bool {
        let nodes = self.guess.len();
        if nodes != self.offset.len() || nodes != self.count.len() || nodes != self.weight.len() {
            return false;
        }
        if self.pattern.len() != self.child.len() {
            return false;
        }
        let e = self.pattern.len();
        for n in 0..nodes {
            let off = self.offset[n] as usize;
            let cnt = self.count[n] as usize;
            if off + cnt > e {
                return false;
            }
            if cnt == 0 && self.weight[n] != 1 {
                return false;
            }
            for i in off..off + cnt {
                let child = self.child[i] as usize;
                if child >= nodes || child <= n {
                    return false;
                }
                if self.pattern[i] as usize >= NUM_PATTERNS {
                    return false;
                }
            }
        }
        true
    }
}
