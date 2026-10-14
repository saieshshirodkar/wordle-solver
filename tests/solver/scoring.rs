use wordle_solver::scoring::{Histogram, ScoreTable};

#[test]
fn entropy_matches_the_closed_form() {
    let t = ScoreTable::entropy(16);
    for c in 1..=16usize {
        #[expect(
            clippy::cast_precision_loss,
            reason = "asserting against the log2 closed form needs f64 math"
        )]
        let want = c as f64 * (c as f64).log2();
        assert!((t.bucket(c) - want).abs() < 1e-12, "bucket {c}");
    }
}

#[test]
fn entropy_is_zero_for_the_empty_bucket() {
    let t = ScoreTable::entropy(4);
    assert_eq!(t.bucket(0), 0.0);
}

#[test]
fn entropy_is_monotonic() {
    let t = ScoreTable::entropy(64);
    for c in 1..64usize {
        assert!(t.bucket(c + 1) >= t.bucket(c), "dip at {c}");
    }
}

#[test]
fn from_lut_preserves_the_supplied_values() {
    let t = ScoreTable::from_lut(vec![0.0, 1.5, 2.5, 3.5]);
    assert_eq!(t.bucket(1), 1.5);
    assert_eq!(t.bucket(3), 3.5);
}

#[test]
fn histogram_tracks_buckets() {
    let mut h = Histogram::new();
    h.add(0);
    h.add(0);
    h.add(5);
    assert_eq!(h.max_bucket(), 2);
    assert_eq!(h.nonzero(), 2);
}

#[test]
fn histogram_reset_clears_everything() {
    let mut h = Histogram::new();
    h.add(3);
    h.add(3);
    h.add(9);
    assert_eq!(h.nonzero(), 2);
    h.reset();
    assert_eq!(h.nonzero(), 0);
    assert_eq!(h.max_bucket(), 0);
    assert_eq!(h.score(&ScoreTable::entropy(4)), 0.0);
}

#[test]
fn histogram_reuse_after_reset_is_correct() {
    let table = ScoreTable::entropy(8);
    let mut h = Histogram::new();
    h.add(1);
    h.add(1);
    let first = h.score(&table);
    h.reset();
    h.add(1);
    h.add(1);
    assert_eq!(h.score(&table), first);
}

#[test]
fn histogram_score_sums_the_occupied_buckets() {
    let table = ScoreTable::from_lut(vec![0.0, 1.0, 2.0, 3.0, 4.0]);
    let mut h = Histogram::new();
    h.add(0);
    h.add(0);
    h.add(0);
    h.add(7);
    h.add(7);
    assert_eq!(h.score(&table), 3.0 + 2.0);
}

#[test]
fn histogram_of_a_single_bucket_is_symmetric() {
    let table = ScoreTable::entropy(8);
    let mut h = Histogram::new();
    h.add(4);
    assert_eq!(h.max_bucket(), 1);
    assert_eq!(h.nonzero(), 1);
    assert_eq!(h.score(&table), table.bucket(1));
}

#[test]
fn histogram_default_matches_new() {
    let a = Histogram::default();
    let b = Histogram::new();
    assert_eq!(a.nonzero(), b.nonzero());
    assert_eq!(a.max_bucket(), b.max_bucket());
    assert_eq!(
        a.score(&ScoreTable::entropy(4)),
        b.score(&ScoreTable::entropy(4))
    );
}

#[test]
fn histogram_saturates_nonzero_at_the_cap() {
    let mut h = Histogram::new();
    for p in 0..=242u8 {
        h.add(p);
    }
    assert_eq!(h.nonzero(), 243);
}
