use wordle_solver::tree::{NONE, Tree};
use wordle_solver::types::NUM_PATTERNS;

use super::{greedy, optimized4};

fn leaf() -> Tree {
    let mut t = Tree::default();
    t.push_node(0, 0, 0, 1);
    t
}

fn two_node() -> Tree {
    let mut t = Tree::default();
    let root = t.push_node(0, 1, 0, 2);
    let child = t.push_node(1, 0, 1, 1);
    assert_eq!(root, 0);
    t.pattern.push(7);
    t.child.push(child);
    t
}

#[test]
fn empty_tree_reports_nothing() {
    let t = Tree::default();
    assert_eq!(t.node_count(), 0);
    assert_eq!(t.edge_count(), 0);
    assert_eq!(t.root_cost(), 0.0);
}

#[test]
fn push_node_assigns_sequential_ids() {
    let mut t = Tree::default();
    assert_eq!(t.push_node(0, 0, 5, 1), 0);
    assert_eq!(t.push_node(1, 0, 6, 1), 1);
    assert_eq!(t.push_node(2, 0, 7, 1), 2);
    assert_eq!(t.node_count(), 3);
}

#[test]
fn leaf_is_terminal_and_costs_one() {
    let t = leaf();
    assert!(t.terminal(0));
    assert_eq!(t.guess_at(0), 0);
    assert_eq!(t.remaining_at(0), 1);
    assert_eq!(t.root_cost(), 1.0);
    assert_eq!(t.costs(), vec![1.0]);
    assert!(t.validate());
}

#[test]
fn two_node_costs_average_over_weights() {
    let t = two_node();
    assert!(!t.terminal(0));
    assert!(t.terminal(1));
    assert_eq!(t.node_count(), 2);
    assert_eq!(t.edge_count(), 1);
    assert!((t.root_cost() - 1.5).abs() < 1e-12);
    assert!(t.validate());
}

#[test]
fn lookup_finds_a_present_pattern() {
    let t = two_node();
    assert_eq!(t.lookup(0, 7), 1);
}

#[test]
fn lookup_returns_none_for_an_absent_pattern() {
    let t = two_node();
    assert_eq!(t.lookup(0, 8), NONE);
    assert_eq!(t.lookup(0, 0), NONE);
}

#[test]
fn lookup_on_a_leaf_always_misses() {
    let t = two_node();
    assert_eq!(t.lookup(1, 7), NONE);
}

#[test]
fn edge_count_tracks_the_pattern_array() {
    let (_, _, _) = greedy();
    let t = optimized4().0.clone();
    assert_eq!(t.edge_count(), t.pattern.len());
    assert_eq!(t.edge_count(), t.child.len());
}

#[test]
fn costs_length_matches_node_count() {
    let t = &optimized4().0;
    assert_eq!(t.costs().len(), t.node_count());
}

#[test]
fn every_node_cost_is_at_least_one() {
    for c in optimized4().0.costs() {
        assert!(c >= 1.0, "cost {c} below one");
    }
}

#[test]
fn costs_decrease_toward_the_leaves() {
    let t = &optimized4().0;
    let cost = t.costs();
    for n in 0..t.node_count() {
        let off = t.offset[n] as usize;
        let cnt = t.count[n] as usize;
        for e in off..off + cnt {
            let child = usize::from(t.child[e]);
            assert!(cost[child] < cost[n], "node {child} not cheaper than {n}");
        }
    }
}

#[test]
fn root_cost_is_the_first_entry() {
    let t = &optimized4().0;
    #[expect(clippy::float_cmp, reason = "identity against the same recurrence")]
    {
        assert_eq!(t.root_cost(), t.costs()[0]);
    }
}

#[test]
fn remaining_weights_sum_to_the_root_weight() {
    let t = &optimized4().0;
    assert_eq!(usize::try_from(t.remaining_at(0)).unwrap_or(0), 2315);

    for n in 0..t.node_count() {
        let node = u16::try_from(n).expect("node index fits u16");
        if t.terminal(node) {
            assert_eq!(t.remaining_at(node), 1);
            continue;
        }
        let off = t.offset[node as usize] as usize;
        let cnt = t.count[node as usize] as usize;
        let total: u64 = (off..off + cnt)
            .map(|e| u64::from(t.remaining_at(t.child[e])))
            .sum();
        assert!(total <= u64::from(t.remaining_at(node)), "node {n}");
        assert!(total > 0, "node {n} has no live children");
    }
}

#[test]
fn validate_accepts_a_built_tree() {
    assert!(optimized4().0.validate());
}

#[test]
fn validate_rejects_mismatched_column_lengths() {
    let mut t = two_node();
    t.offset.push(0);
    assert!(!t.validate());
}

#[test]
fn validate_rejects_mismatched_edge_arrays() {
    let mut t = two_node();
    t.pattern.push(9);
    assert!(!t.validate());
}

#[test]
fn validate_rejects_an_out_of_range_slice() {
    let mut t = two_node();
    t.offset[0] = 99;
    assert!(!t.validate());
}

#[test]
fn validate_rejects_a_terminal_with_the_wrong_weight() {
    let mut t = two_node();
    t.weight[1] = 4;
    assert!(!t.validate());
}

#[test]
fn validate_rejects_a_backwards_child() {
    let mut t = two_node();
    t.child[0] = 0;
    assert!(!t.validate());
}

#[test]
fn validate_rejects_an_out_of_bounds_pattern() {
    let mut t = two_node();
    t.pattern[0] = u8::try_from(NUM_PATTERNS).unwrap_or(0);
    assert!(!t.validate());
}

#[test]
fn debug_is_informative() {
    let d = format!("{:?}", two_node());
    assert!(d.contains("nodes"));
    assert!(d.contains("edges"));
}
