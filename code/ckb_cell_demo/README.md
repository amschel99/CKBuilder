# ckb_cell_demo

A tiny Rust simulation of the **CKB Cell Model** — built to apply my week 1 + week 2 notes
([../../notes/intro.md](../../notes/intro.md), [../../notes/structures.md](../../notes/structures.md)).

It models cells, lock vs type scripts, capacity rules and live → dead consumption, then
runs 6 transactions that each prove one rule (some pass, some get rejected on purpose).

## Run it

From this folder (`code/ckb_cell_demo`):

```bash
cargo run
```

First run compiles, then runs. To just build, or run an optimized release build:

```bash
cargo build            # compile only
cargo run --release    # optimized
```

No dependencies — pure Rust std. Needs Rust + Cargo installed (`rustc --version`).

## What you'll see

| TX  | Rule under test                          | Expected   |
| --- | ---------------------------------------- | ---------- |
| TX1 | Lock script: `2 + 5 ≠ 8`                 | REJECTED   |
| TX2 | Lock script: `3 + 5 = 8`                 | ACCEPTED   |
| TX3 | Min capacity (61 CKByte floor)           | REJECTED   |
| TX4 | Type script: minting `100 → 120`         | REJECTED   |
| TX5 | Type script: split `100 → 60 + 40`       | ACCEPTED   |
| TX6 | Spending an already-dead cell            | REJECTED   |

## Concepts demonstrated

- **Cell = `{ capacity, lock, type, data }`** with `occupied()` enforcing the 61-byte floor
- **Lock script** runs on **inputs only** (ownership)
- **Type script** runs on **inputs + outputs**, grouped by `(code_hash, args)` (state rules / token supply)
- **Capacity balance**: `sum(inputs) ≥ sum(outputs) + fee`
- **Consumption**: validated inputs flip Live → Dead, outputs become new Live cells
