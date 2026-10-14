use wordle_solver::solver::{Secrets, run_bench, secrets, solve_one, walk};
use wordle_solver::types::MAX_GUESSES;

use super::{greedy, optimized4};

#[test]
fn secrets_reports_its_length() {
    let (_, _, answers) = greedy();
    let c = secrets(answers);
    let s = Secrets::new(answers, &c);
    assert_eq!(s.len(), answers.len());
    assert!(!s.is_empty());
}

#[test]
fn empty_secrets_are_empty() {
    let words: &[u64] = &[];
    let c = secrets(words);
    let s = Secrets::new(words, &c);
    assert_eq!(s.len(), 0);
    assert!(s.is_empty());
}

#[test]
fn every_answer_walks_to_a_leaf() {
    let (tree, guesses, answers) = optimized4();
    let c = secrets(answers);
    let s = Secrets::new(answers, &c);
    for i in 0..answers.len() {
        let gc = walk(tree, guesses, &s, i);
        assert!(gc >= 1, "secret {i}");
        assert!(gc <= MAX_GUESSES, "secret {i} needed {gc}");
    }
}

#[test]
fn walking_is_deterministic() {
    let (tree, guesses, answers) = optimized4();
    let c = secrets(answers);
    let s = Secrets::new(answers, &c);
    for i in (0..answers.len()).step_by(37) {
        let first = walk(tree, guesses, &s, i);
        for _ in 0..3 {
            assert_eq!(walk(tree, guesses, &s, i), first, "secret {i}");
        }
    }
}

#[test]
fn a_one_shot_secret_costs_one() {
    let (tree, guesses, answers) = optimized4();
    let c = secrets(answers);
    let s = Secrets::new(answers, &c);
    let cheap: Vec<usize> = (0..answers.len())
        .filter(|&i| walk(tree, guesses, &s, i) == 1)
        .collect();
    assert_eq!(cheap.len(), 1, "expected exactly one 1-guess secret");
}

#[test]
fn solve_one_accepts_valid_indices() {
    let (tree, guesses, answers) = greedy();
    let c = secrets(answers);
    let s = Secrets::new(answers, &c);
    let last = i32::try_from(answers.len() - 1).unwrap_or(0);
    assert!(solve_one(tree, guesses, &s, 0).is_ok());
    assert!(solve_one(tree, guesses, &s, last).is_ok());
}

#[test]
fn solve_one_rejects_bad_indices() {
    let (tree, guesses, answers) = greedy();
    let c = secrets(answers);
    let s = Secrets::new(answers, &c);
    let past_end = i32::try_from(answers.len()).unwrap_or(i32::MAX);
    assert!(solve_one(tree, guesses, &s, -1).is_err());
    assert!(solve_one(tree, guesses, &s, past_end).is_err());
    assert!(solve_one(tree, guesses, &s, i32::MAX).is_err());
}

#[test]
fn solve_one_agrees_with_walk() {
    let (tree, guesses, answers) = optimized4();
    let c = secrets(answers);
    let s = Secrets::new(answers, &c);
    for i in [0usize, 1, 500, 1234, 2314] {
        let via_one = solve_one(tree, guesses, &s, i32::try_from(i).unwrap_or(0)).unwrap_or(-99);
        assert_eq!(via_one, walk(tree, guesses, &s, i), "secret {i}");
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "asserting on f64 metrics requires converting the integer totals"
)]
#[test]
fn benchmark_matches_a_direct_walk() {
    let (tree, guesses, answers) = greedy();
    let c = secrets(answers);
    let s = Secrets::new(answers, &c);
    let b = run_bench(tree, guesses, &s, 1);

    assert_eq!(b.counts.len(), answers.len());
    assert_eq!(b.failed, 0);
    assert_eq!(b.total, b.counts.iter().map(|&x| i64::from(x)).sum::<i64>());
    assert_eq!(b.worst, *b.counts.iter().max().unwrap_or(&0));

    let recomputed = b.total as f64 / answers.len() as f64;
    assert!((recomputed - b.avg).abs() < 1e-9);
    assert!((recomputed - tree.root_cost()).abs() < 1e-9);
}

#[test]
fn benchmark_counts_match_a_walk_one_by_one() {
    let (tree, guesses, answers) = greedy();
    let c = secrets(answers);
    let s = Secrets::new(answers, &c);
    let b = run_bench(tree, guesses, &s, 1);

    for i in (0..answers.len()).step_by(53) {
        assert_eq!(
            usize::try_from(b.counts[i]).unwrap_or(0),
            usize::try_from(walk(tree, guesses, &s, i)).unwrap_or(0),
            "secret {i}"
        );
    }
}

#[test]
fn benchmark_is_thread_count_independent() {
    let (tree, guesses, answers) = greedy();
    let c = secrets(answers);
    let s = Secrets::new(answers, &c);

    let one = run_bench(tree, guesses, &s, 1);
    let many = run_bench(tree, guesses, &s, 16);
    assert_eq!(one.counts, many.counts);
    assert_eq!(one.total, many.total);
    assert_eq!(one.worst, many.worst);
    assert_eq!(one.failed, 0);
    assert_eq!(many.failed, 0);
}

#[test]
fn benchmark_survives_an_oversized_lane_count() {
    let (tree, guesses, answers) = greedy();
    let c = secrets(answers);
    let s = Secrets::new(answers, &c);

    let wide = run_bench(tree, guesses, &s, 100_000);
    assert_eq!(wide.counts.len(), answers.len());
    assert_eq!(wide.failed, 0);
    assert_eq!(wide.total, run_bench(tree, guesses, &s, 1).total);
}

#[test]
fn benchmark_with_no_lanes_falls_back_to_one() {
    let (tree, guesses, answers) = greedy();
    let c = secrets(answers);
    let s = Secrets::new(answers, &c);

    let zero = run_bench(tree, guesses, &s, 0);
    assert_eq!(zero.counts.len(), answers.len());
    assert_eq!(zero.failed, 0);
}

#[test]
fn benchmark_on_empty_input_is_finite() {
    let (tree, guesses, _) = greedy();
    let words: &[u64] = &[];
    let c = secrets(words);
    let s = Secrets::new(words, &c);
    let b = run_bench(tree, guesses, &s, 4);

    assert_eq!(b.counts.len(), 0);
    assert_eq!(b.failed, 0);
    assert!(b.avg.is_finite() && b.avg >= 0.0, "avg {}", b.avg);
    assert!(b.stddev.is_finite(), "stddev {}", b.stddev);
    assert_eq!(b.worst, 0);
    assert_eq!(b.total, 0);
}

#[expect(
    clippy::cast_precision_loss,
    reason = "recomputing the spread needs f64 division"
)]
#[test]
fn benchmark_stddev_matches_the_definition() {
    let (tree, guesses, answers) = greedy();
    let c = secrets(answers);
    let s = Secrets::new(answers, &c);
    let b = run_bench(tree, guesses, &s, 1);

    let mut sq = 0.0f64;
    for &v in &b.counts {
        let d = f64::from(v) - b.avg;
        sq += d * d;
    }
    let want = (sq / answers.len() as f64).sqrt();
    assert!((want - b.stddev).abs() < 1e-9);
}

#[test]
fn benchmark_worst_is_the_slowest_secret() {
    let (tree, guesses, answers) = optimized4();
    let c = secrets(answers);
    let s = Secrets::new(answers, &c);
    let b = run_bench(tree, guesses, &s, 1);
    assert_eq!(b.worst, MAX_GUESSES);
    assert_eq!(b.counts.iter().filter(|&&v| v == b.worst).count(), 1);
}

#[test]
fn default_bench_is_all_zero() {
    let b = wordle_solver::solver::Bench::default();
    assert!(b.counts.is_empty());
    assert_eq!(b.total, 0);
    assert_eq!(b.worst, 0);
    assert_eq!(b.failed, 0);
    assert_eq!(b.avg, 0.0);
    assert_eq!(b.stddev, 0.0);
}
