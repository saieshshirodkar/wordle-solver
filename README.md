# wordle-solver

Solves Wordle optimally using a precomputed greedy-entropy decision tree.

Builds a 28 MB pattern matrix and ~2500-node decision tree at startup (~2.5 s), then solves any game by following indexed children zero computation per guess at solve time.

## Build

```bash
cd build
cmake .. -DCMAKE_BUILD_TYPE=Release
make -j$(nproc)
```

Run from project root:

```bash
./build/wordle-solver
```

## How to use

### Benchmark (default)

Shows stats for all 2315 possible answers:

```
  Average:  3.4652
  Stddev:   0.5864
  Worst:    6
  Total:    0.30 ms
```

### Interactive mode

After benchmark output, type 5-char feedback (g=green, y=yellow, b=gray):

```
  Guess: soare
  Feedback: bgybb
  Guess: glint
```

### CLI mode (one-shot)

Pass feedback as argument to get the next guess:

```bash
./build/wordle-solver bbbyb
glint
```

## Stats

| Metric | |
|---|---|
| Pattern matrix build | ~180 ms |
| Tree build | ~2.3 s |
| Tree nodes | 2488 |
| Solve all 2315 games | ~0.3 ms |
| Average guesses | 3.4652 |
| Worst case | 6 |
| First guess | soare |

## Data

- `data/answers.txt` — 2315 possible answers
- `data/allowed.txt` — 10657 additional valid guesses
