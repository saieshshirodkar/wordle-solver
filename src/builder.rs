use crate::error::{Error, Result};
use crate::pattern::{Counts, letter_counts, pattern};
use crate::scoring::ScoreTable;
use crate::tree::Tree;
use crate::types::{ALL_GREEN, MAX_ANSWERS, NUM_PATTERNS, Word};

mod candidate;
mod partition;
mod scan;
mod value;

pub use candidate::{Candidate, NO_INDEX, candidate_better, threads_available};

const MAX_HIST: usize = NUM_PATTERNS;

pub const MAX_DEPTH: u32 = 64;

#[derive(Debug)]
pub struct Builder {
    answers: Vec<Word>,
    guesses: Vec<Word>,
    counts: Vec<Counts>,
    scores: ScoreTable,
    num_probes: usize,
    is_answer: Vec<bool>,
    levels: Vec<Vec<u16>>,
    offsets: Vec<usize>,
    lengths: Vec<usize>,
    tree: Tree,
}

impl Builder {
    pub fn new(answers: Vec<Word>, probes: Vec<Word>) -> Result<Self> {
        let num_answers = answers.len();
        let num_probes = probes.len();

        if num_answers == 0 {
            return Err(Error::Empty {
                path: "answers".to_owned(),
            });
        }
        if num_answers > MAX_ANSWERS {
            return Err(Error::TooManyAnswers(num_answers));
        }
        if num_probes + num_answers > u16::MAX as usize {
            return Err(Error::TooManyGuesses(num_probes + num_answers));
        }

        let mut guesses = probes;
        guesses.extend_from_slice(&answers);
        let total = guesses.len();

        let counts: Vec<Counts> = answers.iter().copied().map(letter_counts).collect();
        let depth_cap = MAX_DEPTH as usize + 2;

        Ok(Self {
            scores: ScoreTable::entropy(num_answers),
            answers,
            guesses,
            counts,
            num_probes,
            is_answer: vec![false; total],
            levels: (0..depth_cap)
                .map(|_| Vec::with_capacity(num_answers))
                .collect(),
            offsets: vec![0; depth_cap],
            lengths: vec![0; depth_cap],
            tree: Tree::default(),
        })
    }

    #[must_use]
    pub fn guesses(&self) -> &[Word] {
        &self.guesses
    }

    #[must_use]
    pub fn tree(&self) -> &Tree {
        &self.tree
    }

    #[must_use]
    pub fn into_parts(self) -> (Tree, Vec<Word>) {
        (self.tree, self.guesses)
    }

    pub fn build(&mut self) -> Result<()> {
        self.scores = ScoreTable::entropy(self.answers.len());
        self.rebuild()
    }

    fn rebuild(&mut self) -> Result<()> {
        self.tree = Tree::default();
        self.tree.guess.reserve(4096);
        self.tree.offset.reserve(4096);
        self.tree.count.reserve(4096);
        self.tree.weight.reserve(4096);
        self.tree.pattern.reserve(1 << 14);
        self.tree.child.reserve(1 << 14);

        self.tree.push_node(0, 0, 0, 0);

        let n = self.answers.len();
        self.levels[0].clear();
        self.levels[0].extend((0..n).map(|i| i as u16));
        self.offsets[0] = 0;
        self.lengths[0] = n;

        self.build_node(0, 0)
    }

    pub fn optimize(&mut self, rounds: usize) -> Result<Vec<f64>> {
        let mut history = vec![self.expected_guesses()];
        let mut best_tree = self.tree.clone();
        let mut best_avg = history[0];
        for _ in 0..rounds {
            self.scores = Self::fit_value_function(&best_tree, self.answers.len());
            self.rebuild()?;
            let avg = self.expected_guesses();
            history.push(avg);
            if avg + 1e-9 < best_avg {
                best_avg = avg;
                best_tree = self.tree.clone();
            } else {
                break;
            }
        }

        self.scores = Self::fit_value_function(&best_tree, self.answers.len());
        self.tree = best_tree;
        Ok(history)
    }

    fn build_node(&mut self, node: u16, depth: u32) -> Result<()> {
        if depth >= MAX_DEPTH {
            return Err(Error::DepthExceeded { node, depth });
        }

        let level = depth as usize;
        let start = self.offsets[level];
        let len = self.lengths[level];

        if len == 1 {
            let ans = self.levels[level][start];
            self.tree.guess[node as usize] = self.num_probes as u16 + ans;
            self.tree.offset[node as usize] = self.tree.pattern.len() as u32;
            self.tree.count[node as usize] = 0;
            self.tree.weight[node as usize] = 1;
            return Ok(());
        }

        if len == 2 {
            self.split_last_two(node, level, start);
            return Ok(());
        }

        self.mark_answers(level, start, len, true);
        let total = self.guesses.len();
        let threads = self.threads_for(total * len);
        let guess = self.pick(level, node, threads);
        self.mark_answers(level, start, len, false);
        let guess = guess?;

        let (live, nlive) = self.partition(level, start, len, guess);

        let base = self.tree.pattern.len();
        self.tree.pattern.resize(base + nlive, 0);
        self.tree.child.resize(base + nlive, 0);
        for (k, &(p, _, _)) in live.iter().take(nlive).enumerate() {
            self.tree.pattern[base + k] = p;
        }

        let next = level + 1;
        for (k, &(_, c, off)) in live.iter().take(nlive).enumerate() {
            let child = self.tree.push_node(base as u32, 0, 0, c);
            self.offsets[next] = off as usize;
            self.lengths[next] = c as usize;
            self.build_node(child, depth + 1)?;
            self.tree.child[base + k] = child;
        }

        self.tree.guess[node as usize] = guess;
        self.tree.offset[node as usize] = base as u32;
        self.tree.count[node as usize] = nlive as u8;
        self.tree.weight[node as usize] = u32::try_from(len).unwrap_or(u32::MAX);

        Ok(())
    }

    fn split_last_two(&mut self, node: u16, level: usize, start: usize) {
        let first = self.levels[level][start];
        let second = self.levels[level][start + 1];
        let guess = self.num_probes as u16 + first;

        let base = self.tree.pattern.len();
        self.tree.pattern.resize(base + 1, 0);
        self.tree.child.resize(base + 1, 0);

        let child = self.tree.push_node(base as u32, 0, 0, 1);
        self.tree.guess[child as usize] = self.num_probes as u16 + second;
        self.tree.offset[child as usize] = base as u32;
        self.tree.count[child as usize] = 0;

        let gw = self.guesses[usize::from(guess)];
        let i = usize::from(second);
        self.tree.pattern[base] = pattern(gw, self.answers[i], &self.counts[i]);
        self.tree.child[base] = child;

        self.tree.guess[node as usize] = guess;
        self.tree.offset[node as usize] = base as u32;
        self.tree.count[node as usize] = 1;
        self.tree.weight[node as usize] = 2;
    }
}

const _: () = assert!(MAX_HIST == NUM_PATTERNS);
const _: () = assert!(ALL_GREEN < NUM_PATTERNS as u8);
