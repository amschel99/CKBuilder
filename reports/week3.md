# Week 3 Progress


## What I worked on

Week 2 ended on "deploy my own code cell", so that's what I did. Last week's `ckb_cell_demo` only
*simulated* the cell model in Rust. This week I wrote and ran a real one — `#![no_std]` Rust
compiled to a RISC-V binary that runs inside CKB-VM. Most of the time went into the toolchain, not
the code.

### hello_world — first real on-chain script
- Prints `Hello, World!` and returns `0`. Trivial on purpose — the point was the pipeline.
- Built as a RISC-V ELF, ran off-chain in `ckb-debugger` (18,677 cycles), then deployed the binary
  as a code cell on a local devnet and ran it as a type script. Committed at block `0x443`.
- Flipped it to `return 1` and the node rejected the whole tx with `error code 1`.

### secret_lock — started
- Scaffolded only. It'll be a lock that unlocks on the right hash preimage in the witness. No logic
  yet, that's next week.

### Tooling
- Stopped hand-copying files for each new project. `offckb` gives a JS/TS dApp (not what I want);
  the Rust path is `ckb-script-templates` via `cargo-generate`.
- Made a minimal `cargo-generate` template at `code/ckb-template/` so a new contract is one
  command. Commands written up in `code/README.md`.

## Key learnings

### A CKB script isn't a normal Rust program
- `#![no_std]` + `#![no_main]`. `entry!` is a macro (checked the source) that generates `_start`,
  calls my function, exits via syscall `93`. `default_alloc!` gives me a heap.
- Return `i8` IS the verdict: `0` = pass, non-zero = reject the tx. Same 0/non-0 rule from week 1,
  now I'm writing the code that returns it.
- Scripts read the tx through `high_level::load_*`, which wrap the `ckb_load_*` syscalls from last
  week's notes. The `Source` enum is the `CKB_SOURCE_*` stuff made concrete.

### Toolchain gotchas (where the week went)
- ckb-std's default `libc` feature needs riscv gcc/clang or the build dies. Fix:
  `default-features = false`, keep only `allocator` + `dummy-atomic` → pure Rust, links with rust-lld.
- `MemWriteOnFreezedPage` at runtime: CKB-VM freezes executable pages, and my heap shared a page
  with code. Fix is the `ckb-std.ld` linker script splitting code (R+E) and data (R+W) into
  separate page-aligned segments. So the linker file isn't boilerplate — skip it and nothing runs.
- `debug!` is gated behind `debug_assertions` — compiles to nothing in `--release`. Deploy release,
  demo with debug. A live node won't print it regardless.

## Blockers
- Windows toolchain friction (`ckb-debugger` wants gcc + protoc; the freezed-page error cost an
  afternoon). Got through it, just slow.
- Didn't get back to the CCC Playground examples — the script work took over and was more useful.

## Next week
- Write `secret_lock` properly — unlock on the right preimage, reject the wrong one.
- Write a Type Script (small UDT, supply conserved) and place a `WitnessArgs` witness by hand — the
  lock/input_type/output_type split still hasn't clicked from reading.
- Type ID upgradeable deploy, then freeze to a data hash (carried from last week).
