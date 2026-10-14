use wordle_solver::builder::{
    Builder, Candidate, MAX_DEPTH, NO_INDEX, candidate_better, threads_available,
};
use wordle_solver::encode::try_encode;
use wordle_solver::pattern::letter_counts;
use wordle_solver::solver::{Secrets, secrets, walk};
use wordle_solver::types::{ALL_GREEN, MAX_ANSWERS, MAX_GUESSES, Word};

use super::{answers, greedy, optimized4, probes};

fn enc(s: &str) -> Word {
    try_encode(s).unwrap_or(0)
}

#[test]
fn threads_available_is_never_zero() {
    assert!(threads_available() >= 1);
}

#[test]
fn empty_answers_are_rejected() {
    assert!(Builder::new(vec![], vec![]).is_err());
}

#[test]
fn oversized_answers_are_rejected() {
    let too_many = vec![0u64; MAX_ANSWERS + 1];
    assert!(Builder::new(too_many, vec![]).is_err());
}

#[test]
fn oversized_guess_pool_is_rejected() {
    let answers = vec![0u64; 2];
    let probes = vec![0u64; u16::MAX as usize];
    assert!(Builder::new(answers, probes).is_err());
}

#[test]
fn exactly_at_the_answer_cap_is_accepted() {
    let answers = vec![0u64; MAX_ANSWERS];
    let r = Builder::new(answers, vec![]);
    assert!(r.is_ok());
}

#[test]
fn probe_only_pool_is_fine() {
    let mut b =
        Builder::new(vec![enc("soare")], vec![enc("crane")]).unwrap_or_else(|e| panic!("{e}"));
    assert!(b.guesses().contains(&enc("crane")));
    assert!(b.guesses().contains(&enc("soare")));
    b.build().unwrap_or_default();
}

#[test]
fn guesses_are_probes_then_answers() {
    let (_, guesses, answers) = greedy();
    let probes = &guesses[..guesses.len() - answers.len()];
    assert_eq!(probes.len(), 10_657);
    assert_eq!(answers.len(), 2_315);
    assert_eq!(guesses.len(), 12_972);
    for w in answers {
        assert!(guesses.contains(w));
    }
}

#[test]
fn tree_accessor_is_the_owned_tree() {
    let b = Builder::new(vec![enc("soare")], vec![]).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(b.tree().node_count(), 0);
    let (t, g) = b.into_parts();
    assert_eq!(t.node_count(), 0);
    assert_eq!(g.len(), 1);
}

#[test]
fn single_answer_solves_in_one() {
    let one = enc("soare");
    let mut b = Builder::new(vec![one], vec![]).unwrap_or_else(|e| panic!("{e}"));
    b.build().unwrap_or_else(|e| panic!("{e}"));
    let tree = b.tree().clone();
    assert!(tree.validate());
    assert_eq!(tree.node_count(), 1);
    assert!(tree.terminal(0));

    let words = [one];
    let c = secrets(&words);
    let s = Secrets::new(&words, &c);
    assert_eq!(walk(&tree, b.guesses(), &s, 0), 1);
}

#[test]
fn two_answers_build_one_edge() {
    let a = enc("soare");
    let b = enc("crane");
    let mut builder = Builder::new(vec![a, b], vec![]).unwrap_or_else(|e| panic!("{e}"));
    builder.build().unwrap_or_else(|e| panic!("{e}"));
    let tree = builder.tree();
    assert!(tree.validate());
    assert_eq!(tree.node_count(), 2);
    assert_eq!(tree.remaining_at(0), 2);
    assert_eq!(tree.count[0], 1);
    assert_eq!(tree.pattern.len(), 1);
    assert_ne!(tree.pattern[0], ALL_GREEN);

    let guesses = builder.guesses().to_vec();
    let words = vec![a, b];
    let c = secrets(&words);
    let s = Secrets::new(&words, &c);
    assert!(walk(tree, &guesses, &s, 0) <= 2);
    assert!(walk(tree, &guesses, &s, 1) <= 2);
}

#[test]
fn every_answer_solves_within_the_limit() {
    let (tree, guesses, answers) = optimized4();
    let c = secrets(answers);
    let s = Secrets::new(answers, &c);

    let mut worst = 0;
    for i in 0..answers.len() {
        let gc = walk(tree, guesses, &s, i);
        assert!(gc >= 1, "secret {i} unsolved");
        worst = worst.max(gc);
        assert!(gc <= MAX_GUESSES, "secret {i} needed {gc}");
    }
    assert_eq!(worst, MAX_GUESSES, "worst case should be exactly 6");
}

#[test]
fn no_secret_falls_off_the_tree() {
    let (tree, guesses, answers) = optimized4();
    let c = secrets(answers);
    let s = Secrets::new(answers, &c);
    for i in 0..answers.len() {
        assert_ne!(walk(tree, guesses, &s, i), -1, "secret {i} hit a dead edge");
    }
}

#[test]
fn the_root_guess_is_a_known_word() {
    let (tree, guesses, _) = optimized4();
    let first = guesses[tree.guess_at(0) as usize];
    assert!(guesses.contains(&first));
    assert_eq!(wordle_solver::encode::decode(first).len(), 5);
}

#[test]
fn terminal_nodes_always_guess_an_answer() {
    let (tree, guesses, answers) = optimized4();
    let offset = guesses.len() - answers.len();
    for n in 0..tree.node_count() {
        if tree.count[n] == 0 {
            assert!(
                usize::from(tree.guess[n]) >= offset,
                "terminal node {n} guesses a probe word"
            );
        }
    }
}

#[test]
fn building_twice_is_deterministic() {
    let a = answers();
    let p = probes();
    let mut one = Builder::new(a.clone(), p.clone()).unwrap_or_else(|e| panic!("{e}"));
    let mut two = Builder::new(a, p).unwrap_or_else(|e| panic!("{e}"));
    one.build().unwrap_or_default();
    two.build().unwrap_or_default();
    assert_eq!(one.tree().guess, two.tree().guess);
    assert_eq!(one.tree().pattern, two.tree().pattern);
    assert_eq!(one.tree().child, two.tree().child);
}

#[test]
fn a_rebuilt_tree_is_byte_identical() {
    let a = answers();
    let p = probes();
    let mut one = Builder::new(a.clone(), p.clone()).unwrap_or_else(|e| panic!("{e}"));
    let mut two = Builder::new(a, p).unwrap_or_else(|e| panic!("{e}"));
    one.build().unwrap_or_default();
    two.build().unwrap_or_default();

    assert_eq!(one.tree().guess, two.tree().guess);
    assert_eq!(one.tree().pattern, two.tree().pattern);
    assert_eq!(one.tree().child, two.tree().child);
    assert_eq!(one.tree().weight, two.tree().weight);
    assert_eq!(one.tree().offset, two.tree().offset);
}

#[test]
fn answer_order_does_not_change_the_first_guess() {
    let a = answers();
    let p = probes();
    let mut sorted = Builder::new(a.clone(), p.clone()).unwrap_or_else(|e| panic!("{e}"));
    let mut reversed_input = a;
    reversed_input.reverse();
    let mut flipped = Builder::new(reversed_input, p).unwrap_or_else(|e| panic!("{e}"));

    sorted.build().unwrap_or_default();
    flipped.build().unwrap_or_default();
    assert_eq!(
        sorted.tree().guess_at(0),
        flipped.tree().guess_at(0),
        "first guess depends on input order"
    );
}

#[test]
fn optimizer_history_records_every_round_it_tried() {
    let a = answers();
    let p = probes();
    let mut b = Builder::new(a, p).unwrap_or_else(|e| panic!("{e}"));
    b.build().unwrap_or_else(|e| panic!("{e}"));
    let before = b.expected_guesses();
    let history = b.optimize(8).unwrap_or_else(|e| panic!("{e}"));

    assert!(history.len() >= 2);
    assert_eq!(history[0], before);
    assert!(history.iter().all(|v| v.is_finite() && *v >= 1.0));
    let best = history.iter().copied().fold(f64::INFINITY, f64::min);
    assert!(
        best < before - 1e-9,
        "no round beat the greedy tree: best {best}, greedy {before}"
    );
}

#[test]
fn optimizer_history_is_a_trace_not_a_monotone_series() {
    let a = answers();
    let p = probes();
    let mut b = Builder::new(a, p).unwrap_or_else(|e| panic!("{e}"));
    b.build().unwrap_or_else(|e| panic!("{e}"));
    let history = b.optimize(8).unwrap_or_else(|e| panic!("{e}"));

    let last = *history.last().unwrap_or(&0.0);
    let kept = b.expected_guesses();
    if last > kept + 1e-9 {
        assert_eq!(history.len(), 3, "a rejected round should be the final one");
        assert!(history[1] <= kept + 1e-9);
    }
}

#[test]
fn optimizer_never_returns_a_worse_tree() {
    let a = answers();
    let p = probes();
    let mut b = Builder::new(a, p).unwrap_or_else(|e| panic!("{e}"));
    b.build().unwrap_or_else(|e| panic!("{e}"));
    let before = b.expected_guesses();
    b.optimize(8).unwrap_or_else(|e| panic!("{e}"));
    let after = b.expected_guesses();

    assert!(after <= before + 1e-9, "regressed {before} -> {after}");
    assert!(after < before - 1e-4, "gained nothing {before} -> {after}");
    assert!(b.tree().validate());
}

#[test]
fn optimizer_keeps_the_best_round_it_saw() {
    let a = answers();
    let p = probes();
    let mut b = Builder::new(a, p).unwrap_or_else(|e| panic!("{e}"));
    b.build().unwrap_or_else(|e| panic!("{e}"));
    let history = b.optimize(8).unwrap_or_else(|e| panic!("{e}"));

    let best = history.iter().copied().fold(f64::INFINITY, f64::min);
    let kept = b.expected_guesses();
    assert!((kept - best).abs() < 1e-9, "kept {kept}, best was {best}");
}

#[test]
fn optimizer_with_zero_rounds_is_a_no_op() {
    let a = answers();
    let p = probes();
    let mut b = Builder::new(a, p).unwrap_or_else(|e| panic!("{e}"));
    b.build().unwrap_or_else(|e| panic!("{e}"));
    let before = b.tree().guess.clone();
    let history = b.optimize(0).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(history.len(), 1);
    assert_eq!(before, b.tree().guess);
}

#[test]
fn modelled_cost_matches_an_exhaustive_walk() {
    let (tree, guesses, answers) = optimized4();
    let c = secrets(answers);
    let s = Secrets::new(answers, &c);

    let mut sum: i64 = 0;
    for i in 0..answers.len() {
        sum += i64::from(walk(tree, guesses, &s, i));
    }
    #[expect(
        clippy::cast_precision_loss,
        reason = "averaging an integer total over a count needs f64"
    )]
    let measured = sum as f64 / answers.len() as f64;
    let modelled = tree.root_cost();
    assert!(
        (measured - modelled).abs() < 1e-9,
        "weights disagree: measured {measured}, modelled {modelled}"
    );
}

#[test]
fn measured_average_beats_the_greedy_baseline() {
    let (gt, gg, ga) = greedy();
    let gc = secrets(ga);
    let gs = Secrets::new(ga, &gc);
    let mut gsum: i64 = 0;
    for i in 0..ga.len() {
        gsum += i64::from(walk(gt, gg, &gs, i));
    }

    let (ot, og, oa) = optimized4();
    let oc = secrets(oa);
    let os = Secrets::new(oa, &oc);
    let mut osum: i64 = 0;
    for i in 0..oa.len() {
        osum += i64::from(walk(ot, og, &os, i));
    }

    assert!(
        osum < gsum,
        "optimized {osum} not better than greedy {gsum}"
    );
}

#[test]
fn optimizer_improves_the_greedy_average() {
    let a = answers();
    let p = probes();
    let mut b = Builder::new(a, p).unwrap_or_else(|e| panic!("{e}"));
    b.build().unwrap_or_else(|e| panic!("{e}"));
    let before = b.expected_guesses();
    b.optimize(8).unwrap_or_else(|e| panic!("{e}"));
    assert!(b.expected_guesses() < before);
}

#[test]
fn depth_cap_is_a_real_limit() {
    assert!(MAX_DEPTH >= MAX_GUESSES as u32);
    assert!(MAX_DEPTH > MAX_GUESSES as u32, "no headroom for safety");
}

#[test]
fn a_large_sample_still_solves() {
    let all = probes();
    let step = all.len() / 1500;
    let sample: Vec<Word> = all.iter().step_by(step).copied().collect();
    assert!(sample.len() > 1000);

    let mut b = Builder::new(sample.clone(), vec![]).unwrap_or_else(|e| panic!("{e}"));
    b.build().unwrap_or_else(|e| panic!("{e}"));
    let tree = b.tree().clone();
    assert!(tree.validate());

    let guesses = b.guesses().to_vec();
    let counts = secrets(&sample);
    let s = Secrets::new(&sample, &counts);
    for i in 0..sample.len() {
        let gc = walk(&tree, &guesses, &s, i);
        assert!(gc >= 1, "secret {i} unsolved");
        assert!(gc <= MAX_GUESSES + 2, "secret {i} needed {gc}");
    }
}

#[test]
fn letter_counts_cover_every_answer() {
    let a = answers();
    assert_eq!(letter_counts(enc("soare")).len(), 26);
    assert_eq!(a.len(), 2315);
}

#[test]
fn candidate_perfect_needs_an_answer_that_isolates() {
    let isolated = Candidate {
        selection: 1.0,
        max_bucket: 1,
        nonzero: 5,
        index: 3,
        is_answer: true,
    };
    assert!(isolated.perfect());

    let not_answer = Candidate {
        is_answer: false,
        ..isolated
    };
    assert!(!not_answer.perfect());

    let wide = Candidate {
        max_bucket: 2,
        ..isolated
    };
    assert!(!wide.perfect());

    let empty = Candidate::default();
    assert!(!empty.perfect());
}

#[test]
fn candidate_default_is_the_worst_possible() {
    let d = Candidate::default();
    assert_eq!(d.index, NO_INDEX);
    assert_eq!(d.max_bucket, 0);
    assert_eq!(d.nonzero, 0);
    assert!(!d.is_answer);
    assert!(d.selection.is_infinite());
}

#[test]
fn a_valid_candidate_always_beats_the_empty_one() {
    let good = Candidate {
        selection: 99.0,
        max_bucket: 99,
        nonzero: 0,
        index: 0,
        is_answer: false,
    };
    let empty = Candidate::default();
    assert!(candidate_better(&good, &empty));
    assert!(!candidate_better(&empty, &good));
    assert!(!candidate_better(&empty, &empty));
}

#[test]
fn candidate_ordering_is_strict_and_total() {
    let base = Candidate {
        selection: 10.0,
        max_bucket: 5,
        nonzero: 3,
        index: 0,
        is_answer: false,
    };
    let same = base;
    assert!(!candidate_better(&base, &same));
    assert!(!candidate_better(&same, &base));

    let cheaper = Candidate {
        selection: 9.0,
        ..base
    };
    assert!(candidate_better(&cheaper, &base));
    assert!(!candidate_better(&base, &cheaper));

    let dearer = Candidate {
        selection: 11.0,
        ..base
    };
    assert!(!candidate_better(&dearer, &base));

    let is_answer = Candidate {
        is_answer: true,
        ..base
    };
    assert!(candidate_better(&is_answer, &base));

    let smaller_bucket = Candidate {
        max_bucket: 4,
        ..base
    };
    assert!(candidate_better(&smaller_bucket, &base));

    let more_buckets = Candidate { nonzero: 4, ..base };
    assert!(candidate_better(&more_buckets, &base));

    let later_index = Candidate { index: 1, ..base };
    assert!(candidate_better(&base, &later_index));
}

#[test]
fn a_perfect_candidate_dominates_everything() {
    let perfect = Candidate {
        selection: 1e9,
        max_bucket: 1,
        nonzero: 200,
        index: 50,
        is_answer: true,
    };
    let cheap_but_weak = Candidate {
        selection: 0.0,
        max_bucket: 900,
        nonzero: 2,
        index: 1,
        is_answer: false,
    };
    assert!(candidate_better(&perfect, &cheap_but_weak));
    assert!(!candidate_better(&cheap_but_weak, &perfect));
}

#[test]
fn ties_below_the_epsilon_are_broken_by_index() {
    let a = Candidate {
        selection: 100.0,
        max_bucket: 5,
        nonzero: 3,
        index: 4,
        is_answer: false,
    };
    let b = Candidate {
        selection: 100.0 + 1e-15,
        max_bucket: 5,
        nonzero: 3,
        index: 9,
        is_answer: false,
    };
    assert!(candidate_better(&a, &b));
    assert!(!candidate_better(&b, &a));
}
