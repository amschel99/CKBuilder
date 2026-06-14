# hello_world — a real CKB on-chain script

Unlike [../ckb_cell_demo/](../ckb_cell_demo/) (which *simulates* CKB on the host), this is an
**actual CKB script**: `#![no_std]` Rust compiled to a RISC-V ELF binary that runs inside
**CKB-VM**. It is the genuine artifact you would deploy on-chain.

All it does is print `Hello, World!` via the CKB debug syscall and return `0` (success).
We deployed it to a local devnet, ran it as a cell's type script (committed at block `0x443`),
and proved a `return 1` variant gets rejected with `error code 1`.

---

## Project layout

```
hello_world/
├── Cargo.toml            # package + ckb-std dependency + release profile
├── .cargo/config.toml    # default RISC-V target + linker flag
├── ckb-std.ld            # linker script (memory layout)
└── src/main.rs           # the script
```

A normal Rust project has neither `.cargo/config.toml` nor `ckb-std.ld`. Those two files are
what turn a plain crate into a CKB-VM binary.

## Prerequisite

```bash
rustup target add riscv64imac-unknown-none-elf   # CKB-VM's instruction set
```

## File-by-file

### `Cargo.toml`
```toml
[dependencies]
ckb-std = { version = "0.16", default-features = false, features = ["allocator", "dummy-atomic"] }

[profile.release]
opt-level = "z"      # optimize for SIZE — on CKB, bytes = capacity = money
lto = true
codegen-units = 1
panic = "abort"      # no unwinding in no_std
strip = false
```
**Why `default-features = false`:** ckb-std's default `libc` feature compiles C with a RISC-V
C compiler (`riscv64-unknown-elf-gcc` / `clang`). If you don't have one the build dies with
`program not found`. Keeping only `allocator` (heap for `default_alloc!`) and `dummy-atomic`
(riscv atomics shim) stays pure-Rust and links with `rust-lld`.

### `.cargo/config.toml`
```toml
[build]
target = "riscv64imac-unknown-none-elf"          # so plain `cargo build` cross-compiles

[target.riscv64imac-unknown-none-elf]
rustflags = ["-C", "link-arg=-Tckb-std.ld"]      # use our linker script
```

### `ckb-std.ld`
Splits code and data into separate page-aligned segments via `PHDRS`:
```ld
PHDRS {
    code PT_LOAD FLAGS(5);  /* R+E */
    data PT_LOAD FLAGS(6);  /* R+W */
}
```
**Why this matters:** without separate segments + `. = ALIGN(0x1000)`, the writable `.bss`/heap
shares a page with executable code. CKB-VM freezes executable pages, so the heap write aborts
with `VM Internal Error: MemWriteOnFreezedPage`. We hit exactly this — the `PHDRS` split is the fix.

### `src/main.rs`
```rust
#![no_std]
#![no_main]
ckb_std::entry!(program_entry);   // generates _start + panic handler
ckb_std::default_alloc!();        // heap allocator

pub fn program_entry() -> i8 {
    debug!("Hello, World!");
    0   // exit code: 0 = pass (unlock / allow), non-zero = reject the whole tx
}
```

---

## Build

```bash
cargo build --release   # what you DEPLOY (small, silent) -> target/.../release/hello_world
cargo build             # DEBUG build (what the debugger needs to print) -> target/.../debug/hello_world
```

`file target/riscv64imac-unknown-none-elf/release/hello_world` → `ELF 64-bit LSB executable, UCB RISC-V`.

> **THE big gotcha:** `ckb_std::debug!` is gated behind `debug_assertions`. In a `--release`
> build it compiles to **nothing** — no output anywhere. You only see `Hello, World!` from a
> **debug** build. So: deploy release, demo with debug.

## Run off-chain (ckb-debugger)

```bash
cargo install ckb-debugger        # one-time (on Windows needs gcc + protoc on PATH)
ckb-debugger --bin target/riscv64imac-unknown-none-elf/debug/hello_world
```
```
Script log: Hello, World!
Run result: 0
All cycles: 18677
```

## Run on a local devnet (the real chain)

Using the `ckb` node + `ckb-cli` (a devnet was set up under `C:\Users\ADMIN\ckb-node\devnet`):

```bash
# 1. deploy the binary as a code cell (data = the ELF), owned by your account
ckb-cli wallet transfer --privkey-path miner.key --to-address <addr> \
        --capacity 15000 --to-data-path <binary> --fee-rate 1000
# -> note the tx hash; the code cell's data hash is its type-script code_hash

# 2. build a tx whose OUTPUT carries a type script pointing at that code cell,
#    plus a cell_dep to the code cell. (ckb-cli tx can't add a type script via flags,
#    so we inject it into the tx JSON, then:)
ckb-cli tx sign-inputs --privkey-path miner.key --tx-file run.json --add-signatures
ckb-cli tx send --tx-file run.json
```

The node runs the type script during verification: return `0` → tx committed on-chain;
return non-zero → `TransactionFailedToVerify ... ValidationFailure: error code N`.
(Note: a live node does **not** print `debug!` output — that's a debugger/test-only feature.)

## How it differs from a normal Rust program

- `#![no_std]` / `#![no_main]` — no standard library, no OS, no `main()`
- `ckb_std::entry!` provides the real `_start` (reads argc/argv, calls our entry, exits via syscall 93)
- `debug!` is the `ckb_debug` syscall (2177), not `println!` — and only active in debug builds
- the return `i8` is the script's **exit code**: `0` = success/unlock, non-zero = reject
- target is `riscv64imac-unknown-none-elf` (CKB-VM's ISA), linked with [ckb-std.ld](ckb-std.ld)

## For production

The idiomatic scaffold is `ckb-script-templates` (via `cargo-generate`), which adds a test
harness (`ckb-testtool`) and a Makefile. That path needs `clang` + `make`; the hand-rolled
setup here avoids both and builds with just `rustup` + `rust-lld`.
```
