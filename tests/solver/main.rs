#![allow(
    clippy::float_cmp,
    reason = "tests assert exact equality on values chosen to be representable"
)]

mod builder;
mod cache;
mod encode;
mod error;
mod fmt;
mod pattern;
mod scoring;
mod solver;
mod tree;
mod types;
mod wordlist;

use std::sync::OnceLock;

use wordle_solver::builder::Builder;
use wordle_solver::tree::Tree;
use wordle_solver::types::Word;
use wordle_solver::wordlist::load_words;

pub type Fixture = (Tree, Vec<Word>, Vec<Word>);

fn answers() -> Vec<Word> {
    load_words("data/answers.txt").unwrap_or_default()
}

fn probes() -> Vec<Word> {
    load_words("data/guesses.txt").unwrap_or_default()
}

fn build_rounds(rounds: Option<usize>) -> Fixture {
    let (a, p) = (answers(), probes());
    let mut b = Builder::new(a.clone(), p).unwrap_or_else(|e| panic!("{e}"));
    b.build().unwrap_or_else(|e| panic!("{e}"));
    if let Some(n) = rounds {
        b.optimize(n).unwrap_or_else(|e| panic!("{e}"));
    }
    (b.tree().clone(), b.guesses().to_vec(), a)
}

pub fn greedy() -> &'static Fixture {
    static F: OnceLock<Fixture> = OnceLock::new();
    F.get_or_init(|| build_rounds(None))
}

pub fn optimized2() -> &'static Fixture {
    static F: OnceLock<Fixture> = OnceLock::new();
    F.get_or_init(|| build_rounds(Some(2)))
}

pub fn optimized4() -> &'static Fixture {
    static F: OnceLock<Fixture> = OnceLock::new();
    F.get_or_init(|| build_rounds(Some(4)))
}

#[must_use]
pub fn temp(name: &str) -> String {
    std::env::temp_dir()
        .join(name)
        .to_string_lossy()
        .into_owned()
}
