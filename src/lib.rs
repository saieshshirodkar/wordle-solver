#![expect(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    reason = "widths are bounded by Builder::new caps"
)]

pub mod builder;
pub mod cache;
pub mod encode;
pub mod error;
pub mod fmt;
pub mod pattern;
pub mod scoring;
pub mod solver;
pub mod tree;
pub mod types;
pub mod wordlist;
