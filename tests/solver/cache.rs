use wordle_solver::cache::{Meta, hash_lists, load, save};
use wordle_solver::solver::{secrets, walk};

use super::{optimized2, optimized4, temp};

fn roundtrip(rounds: usize) -> &'static (wordle_solver::tree::Tree, Vec<u64>, Vec<u64>) {
    if rounds == 2 {
        optimized2()
    } else {
        optimized4()
    }
}

#[test]
fn hash_is_stable_for_the_same_input() {
    let a = vec![1u64, 2, 3];
    let b = vec![4u64, 5];
    assert_eq!(hash_lists(&a, &b), hash_lists(&a, &b));
}

#[test]
fn hash_separates_different_lists() {
    assert_ne!(hash_lists(&[1, 2], &[3]), hash_lists(&[1, 3], &[2]));
    assert_ne!(hash_lists(&[1], &[2]), hash_lists(&[2], &[1]));
    assert_ne!(hash_lists(&[1, 2], &[]), hash_lists(&[2], &[1]));
}

#[test]
fn hash_only_covers_the_concatenation() {
    assert_eq!(
        hash_lists(&[1, 2, 3], &[4]),
        hash_lists(&[1, 2], &[3, 4]),
        "the split between probes and answers is not hashed; Meta::matches covers it"
    );
    assert_eq!(
        hash_lists(&[1], &[]),
        hash_lists(&[], &[1]),
        "the split is not hashed"
    );
}

#[test]
fn hash_responds_to_a_single_changed_word() {
    let base: Vec<u64> = (1..100).collect();
    let mut changed = base.clone();
    changed[42] = 999;
    assert_ne!(hash_lists(&base, &[]), hash_lists(&changed, &[]));
}

#[test]
fn meta_records_its_inputs() {
    let m = Meta::new(10657, 2315, 6, 7949);
    assert_eq!(usize::try_from(m.num_probes).unwrap_or(0), 10657);
    assert_eq!(usize::try_from(m.num_answers).unwrap_or(0), 2315);
    assert_eq!(m.worst, 6);
    assert_eq!(m.total, 7949);
    assert_eq!(m.list_hash, 0);
}

#[test]
fn meta_matches_only_on_hash_and_counts() {
    let mut m = Meta::new(10, 20, 6, 100);
    m.list_hash = 0xDEAD_BEEF;
    assert!(m.matches(0xDEAD_BEEF, 10, 20));
    assert!(!m.matches(0x1234, 10, 20));
    assert!(!m.matches(0xDEAD_BEEF, 11, 20));
    assert!(!m.matches(0xDEAD_BEEF, 10, 21));
}

#[test]
fn meta_matches_ignores_worst_and_total() {
    let a = Meta::new(10, 20, 6, 100);
    let b = Meta::new(10, 20, 1, 1);
    assert!(a.matches(b.list_hash, 10, 20));
    assert!(b.matches(a.list_hash, 10, 20));
}

#[test]
fn default_meta_is_empty() {
    let m = Meta::default();
    assert_eq!(m.list_hash, 0);
    assert_eq!(m.num_probes, 0);
    assert_eq!(m.num_answers, 0);
    assert_eq!(m.worst, 0);
    assert_eq!(m.total, 0);
}

#[test]
fn roundtrip_is_lossless() {
    const WORST: i32 = 6;
    const TOTAL: i64 = 8020;

    let (tree, guesses, answers) = optimized2();
    let probes = &guesses[..guesses.len() - answers.len()];
    let hash = hash_lists(probes, answers);
    let mut meta = Meta::new(probes.len(), answers.len(), WORST, TOTAL);
    meta.list_hash = hash;
    let p = temp("wordle-rs-cache-roundtrip.bin");

    save(&p, tree, &meta).unwrap_or_else(|e| panic!("{e}"));
    let (loaded, back) = load(&p, hash, probes.len(), answers.len())
        .unwrap_or_else(|e| panic!("{e}"))
        .unwrap_or_else(|| panic!("cache did not load"));
    let _ = std::fs::remove_file(&p);

    assert_eq!(loaded.guess, tree.guess);
    assert_eq!(loaded.offset, tree.offset);
    assert_eq!(loaded.count, tree.count);
    assert_eq!(loaded.pattern, tree.pattern);
    assert_eq!(loaded.child, tree.child);
    assert_eq!(loaded.weight, tree.weight);
    assert_eq!(back.list_hash, hash);
    assert_eq!(back.worst, WORST);
    assert_eq!(back.total, TOTAL);
    assert_eq!(usize::try_from(back.num_probes).unwrap_or(0), probes.len());
    assert_eq!(
        usize::try_from(back.num_answers).unwrap_or(0),
        answers.len()
    );
}

#[test]
fn a_loaded_tree_solves_identically() {
    let (tree, guesses, answers) = optimized2();
    let probes = &guesses[..guesses.len() - answers.len()];
    let hash = hash_lists(probes, answers);
    let mut meta = Meta::new(probes.len(), answers.len(), 6, 8020);
    meta.list_hash = hash;
    let p = temp("wordle-rs-cache-equiv.bin");

    save(&p, tree, &meta).unwrap_or_default();
    let (loaded, _) = load(&p, hash, probes.len(), answers.len())
        .unwrap_or_default()
        .unwrap_or_else(|| panic!("load"));
    let _ = std::fs::remove_file(&p);

    let c = secrets(answers);
    let s = wordle_solver::solver::Secrets::new(answers, &c);
    for i in 0..answers.len() {
        assert_eq!(
            walk(tree, guesses, &s, i),
            walk(&loaded, guesses, &s, i),
            "cached tree diverged at {i}"
        );
    }
}

#[test]
fn a_missing_file_is_not_an_error() {
    let r = load(&temp("wordle-rs-definitely-absent.bin"), 0, 0, 0);
    assert!(matches!(r, Ok(None)));
}

#[test]
fn a_short_file_is_rejected() {
    let p = temp("wordle-rs-cache-short.bin");
    std::fs::write(&p, b"WDLR4").unwrap_or_default();
    assert!(load(&p, 0, 0, 0).unwrap_or_default().is_none());
    let _ = std::fs::remove_file(&p);
}

#[test]
fn garbage_is_rejected() {
    let p = temp("wordle-rs-cache-garbage.bin");
    std::fs::write(&p, b"not a wordle tree at all, just bytes").unwrap_or_default();
    assert!(load(&p, 0, 0, 0).unwrap_or_default().is_none());
    let _ = std::fs::remove_file(&p);
}

#[test]
fn a_bad_magic_is_rejected() {
    let p = temp("wordle-rs-cache-magic.bin");
    std::fs::write(&p, b"XXXXXXXXXXXX").unwrap_or_default();
    assert!(load(&p, 0, 0, 0).unwrap_or_default().is_none());
    let _ = std::fs::remove_file(&p);
}

fn header_with(version: u32, nodes: u32, edges: u32, hash: u64) -> Vec<u8> {
    let mut h = b"WDLR4\0\0\0".to_vec();
    h.resize(56, 0);
    h[8..12].copy_from_slice(&version.to_le_bytes());
    h[12..16].copy_from_slice(&nodes.to_le_bytes());
    h[16..20].copy_from_slice(&edges.to_le_bytes());
    h[36..44].copy_from_slice(&hash.to_le_bytes());
    h
}

#[test]
fn a_version_mismatch_is_rejected() {
    let p = temp("wordle-rs-cache-version.bin");
    std::fs::write(&p, header_with(4, 1, 0, 0)).unwrap_or_default();
    assert!(load(&p, 0, 0, 0).unwrap_or_default().is_none());
    let _ = std::fs::remove_file(&p);
}

#[test]
fn a_zero_node_count_is_rejected() {
    let p = temp("wordle-rs-cache-nodes.bin");
    std::fs::write(&p, header_with(5, 0, 0, 0)).unwrap_or_default();
    assert!(load(&p, 0, 0, 0).unwrap_or_default().is_none());
    let _ = std::fs::remove_file(&p);
}

#[test]
fn an_absurd_node_count_is_rejected() {
    let p = temp("wordle-rs-cache-absurd.bin");
    std::fs::write(&p, header_with(5, u32::MAX, 0, 0)).unwrap_or_default();
    assert!(load(&p, 0, 0, 0).unwrap_or_default().is_none());
    let _ = std::fs::remove_file(&p);
}

#[test]
fn a_truncated_body_is_rejected() {
    let p = temp("wordle-rs-cache-truncated.bin");
    std::fs::write(&p, header_with(5, 10, 10, 0)).unwrap_or_default();
    assert!(load(&p, 0, 0, 0).unwrap_or_default().is_none());
    let _ = std::fs::remove_file(&p);
}

#[test]
fn a_wrong_hash_is_rejected() {
    let (tree, guesses, answers) = optimized2();
    let probes = &guesses[..guesses.len() - answers.len()];
    let hash = hash_lists(probes, answers);
    let mut meta = Meta::new(probes.len(), answers.len(), 6, 8020);
    meta.list_hash = hash;
    let p = temp("wordle-rs-cache-hash.bin");

    save(&p, tree, &meta).unwrap_or_default();
    assert!(
        load(&p, 0, probes.len(), answers.len())
            .unwrap_or_default()
            .is_none()
    );
    assert!(
        load(&p, hash, probes.len() + 1, answers.len())
            .unwrap_or_default()
            .is_none()
    );
    assert!(
        load(&p, hash, probes.len(), answers.len() + 1)
            .unwrap_or_default()
            .is_none()
    );
    let _ = std::fs::remove_file(&p);
}

#[test]
fn a_structurally_invalid_body_is_rejected() {
    let (tree, guesses, answers) = optimized2();
    let probes = &guesses[..guesses.len() - answers.len()];
    let hash = hash_lists(probes, answers);
    let mut meta = Meta::new(probes.len(), answers.len(), 6, 8020);
    meta.list_hash = hash;
    let p = temp("wordle-rs-cache-invalid.bin");

    save(&p, tree, &meta).unwrap_or_default();

    let mut bytes = std::fs::read(&p).unwrap_or_default();
    let off = 56 + 6;
    bytes[off..off + 2].copy_from_slice(&0u16.to_le_bytes());
    std::fs::write(&p, &bytes).unwrap_or_default();

    assert!(
        load(&p, hash, probes.len(), answers.len())
            .unwrap_or_default()
            .is_none()
    );
    let _ = std::fs::remove_file(&p);
}

#[test]
fn save_and_load_handle_an_empty_tree() {
    let hash = 42u64;
    let meta = Meta::new(0, 1, 0, 0);
    let p = temp("wordle-rs-cache-empty.bin");
    save(&p, &wordle_solver::tree::Tree::default(), &meta).unwrap_or_default();
    let r = load(&p, hash, 0, 1).unwrap_or_default();
    assert!(r.is_none(), "an empty tree must not roundtrip");
    let _ = std::fs::remove_file(&p);
}

#[test]
fn round_trip_helper_selects_the_right_fixture() {
    assert_eq!(roundtrip(2).0.node_count(), optimized2().0.node_count());
    assert_eq!(roundtrip(4).0.node_count(), optimized4().0.node_count());
}
