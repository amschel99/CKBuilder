# CKB Smart Contracts

On-chain scripts for Nervos CKB, written in Rust against [`ckb-std`](https://crates.io/crates/ckb-std).
Each project compiles to a single RISC-V binary that runs inside the CKB-VM.

## Prerequisites (one-time setup)

```powershell
# 1. Rust (if not already installed) — https://rustup.rs
#    Then add the RISC-V bare-metal target the CKB-VM uses:
rustup target add riscv64imac-unknown-none-elf
```

That's all you need to build. `cargo` ships the LLVM linker (`rust-lld`), so no
extra toolchain is required.

## Start a new project

Use the scaffolder. It creates a project under `code/` with the full layout
(`Cargo.toml`, `.cargo/config.toml`, `ckb-std.ld`, `src/main.rs`) and, with
`-Build`, compiles it immediately.

```powershell
# from the repo root (c:\Users\ADMIN\CKBuilder)

# scaffold only
.\code\new-ckb-contract.ps1 my_contract

# scaffold AND build in one step
.\code\new-ckb-contract.ps1 my_contract -Build

# pin a specific ckb-std version (default is 0.16)
.\code\new-ckb-contract.ps1 my_contract -CkbStdVersion 1.1.0 -Build
```

Notes:
- Dashes in the name become underscores for the Rust crate (`my-contract` -> crate `my_contract`); the folder keeps the name you passed.
- The script refuses to overwrite an existing directory.

## Build an existing project

```powershell
cd code\my_contract
cargo build --release
```

The compiled binary lands at:

```
target\riscv64imac-unknown-none-elf\release\my_contract
```

A minimal contract is ~11 KB. The `release` profile is tuned for size
(`opt-level = "z"`, `lto = true`, `panic = "abort"`).

## Inspect the output

```powershell
# size of the compiled script
(Get-Item .\target\riscv64imac-unknown-none-elf\release\my_contract).Length

# blake2b-256 hash = the script's code_hash on-chain (needs ckb-cli installed)
ckb-cli util blake2b --binary-path .\target\riscv64imac-unknown-none-elf\release\my_contract
```

## See debug output

The `debug!` macro is stripped from release builds. To keep it for debugging:

```powershell
$env:RUSTFLAGS = "--cfg debug_assertions"
cargo build --release
$env:RUSTFLAGS = ""   # reset afterwards
```

To actually *run* the binary and see the output, use
[`ckb-debugger`](https://github.com/nervosnetwork/ckb-standalone-debugger):

```powershell
ckb-debugger --bin .\target\riscv64imac-unknown-none-elf\release\my_contract
```

## Project layout

```
code\my_contract\
  .cargo\config.toml   # target = riscv64imac-unknown-none-elf + -Tckb-std.ld
  .gitignore           # /target
  Cargo.toml           # ckb-std dep + size-optimized release profile
  ckb-std.ld           # linker script: memory layout for the CKB-VM
  src\main.rs          # entry!(program_entry) + default_alloc! + your logic
```

## Existing projects

- `hello_world\` — minimal script: prints a debug message, returns 0.
- `secret_lock\` — work in progress.

## Writing contract logic

The stub in `src/main.rs` just returns `0` (success). Real scripts read the
transaction via `ckb_std::high_level`:

```rust
use ckb_std::high_level::{load_script, load_cell_data};
use ckb_std::ckb_constants::Source;
```

To use `high_level` / `ckb_types`, enable the `ckb-types` feature in
`Cargo.toml` (the scaffold ships with only `allocator` + `dummy-atomic`):

```toml
ckb-std = { version = "0.16", default-features = false, features = ["allocator", "dummy-atomic", "ckb-types"] }
```

## Note on testing

This layout is hand-rolled and has **no test harness**. For unit tests with a
mock CKB context (`ckb-testtool`), generate a project with the official
[`ckb-script-templates`](https://github.com/cryptape/ckb-script-templates):

```powershell
cargo install cargo-generate
cargo generate gh:cryptape/ckb-script-templates workspace
```
