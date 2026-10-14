# wordle-solver

Solves Wordle optimally using a precomputed entropy decision tree.

Builds a ~2500-node decision tree at startup (~2.2 s), then solves any game by following indexed children — zero computation per guess at solve time.

## Build

```bash
cargo build --release
```

Run from project root:

```bash
./target/release/wordle-solver
```

## How to use

### Benchmark (default)

Shows stats for all 2315 possible answers:

```
answers:   2315
probes:    10657
candidates: 12972
first:     trace

distribution: 1:1 2:74 3:1219 4:963 5:57 6:1

results
  average:  3.4337
  stddev:   0.6033
  worst:    6
  sum:      7949  solve all: 1.616 ms
```

### Interactive mode

After benchmark output, type 5-char feedback (g=green, y=yellow, b=gray):

```
  guess: trace
  feedback: bgybb
  guess: arbor
```

### CLI mode (one-shot)

Pass feedback as argument to get the next guess:

```bash
./target/release/wordle-solver bbbyb
muils
```

### Options

```
--answers PATH    answer list (default data/answers.txt)
--allowed PATH    extra guess list (default data/guesses.txt)
--cache PATH      tree cache file (default wordle-tree.bin)
--no-cache        skip the cache, always rebuild
--no-bench        skip the benchmark
--rounds N        optimizer rounds (default 6)
```

## How it works

A greedy pass builds the tree by Shannon entropy: at each node, try every candidate word and keep the one whose feedback splits the remaining answers most evenly.

That alone averages 3.4644. The solver then runs policy iteration — measure exact cost-to-go at every node, fit a value function from those measurements, rebuild against it, repeat. Round 2 reaches 3.4337; round 3 regresses and the climb stops, keeping the best tree it saw.

No dependencies, and `unsafe` is forbidden at the crate level.

## Stats

| Metric | |
|---|---|
| Tree build | ~2.2 s |
| Tree nodes | 2478 |
| Solve all 2315 games | ~1.5–2.5 ms |
| Average guesses | 3.4337 |
| Worst case | 6 |
| First guess | trace |
| Peak memory | 4.3 MB |
| Tests | 170 |

The tree is cached to `wordle-tree.bin` (34 KB) on first run, so later runs load in ~0.1 ms. The cache is keyed on a hash of both word lists, so editing either file rebuilds automatically.

## Tests

```bash
cargo test --release
```

166 integration tests plus 4 unit tests, ~9 s. Coverage mirrors the source layout — one test module per source module.

## Data

- `data/answers.txt` — 2315 possible answers
- `data/guesses.txt` — 10657 additional valid guesses