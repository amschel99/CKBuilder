# week3 :: actually running a script onchain (my own words)

week1 was theory, week2 was structure + a fake simulation in rust. this week i finally ran a REAL
one. and 80% of the week was fighting the build not writing code lol. writing the pain down so i
dont step on the same rakes again.

the mental shift: last week's ckb_cell_demo was me pretending to be the vm in rust on my laptop.
this week the actual risc-v binary runs INSIDE ckb-vm. totally different thing. the demo helped me
understand it but it was never going onchain.

---

## a ckb script is not a normal rust program

top of the file:
#![no_std]   -> no standard library. no Vec/String unless i bring an allocator myself.
#![no_main]  -> no main(). i provide the entry point.

ckb_std::entry!(program_entry)  <- this is a MACRO not a module (went and read the source to be
sure). it generates the real `_start`. what _start does:
  - reads argc/argv off the stack
  - calls my program_entry()
  - exits via syscall 93 (SYS_EXIT)
it also installs the panic handler. so the macro is basically the tiny runtime between the vm and
my function.

ckb_std::default_alloc!()  <- sets up a buddy allocator (4kb fixed + 516kb dynamic heap by
default). without this, no heap, no alloc types.

program_entry() -> i8   <- the i8 is the EXIT CODE and thats the whole verdict.
  return 0   = pass (unlock / allow the tx)
  return !=0 = reject the entire tx
this is literally the 0/non-0 thing from week1, except now im the one writing the code that returns
it. cool to see it close the loop.

> eth: no equivalent to no_std/_start, you write solidity and the evm wraps it. here im basically
> writing bare metal, the script is a freestanding risc-v program.
> btc: btc script cant even do this, its a fixed opcode stack. a ckb script is a full program. this
> is the entire reason ckb is more powerful.

---

## the load_* syscalls (the only window the script has)

a script cant see anything except thru ckb_std::high_level::load_* which wrap the ckb_load_*
syscalls i wrote about last week. eg load_script() to read myself, load_cell_data(i, source),
load_witness(i, source). the `source` is that Source enum: Input / Output / CellDep / GroupInput /
GroupOutput etc. so all the CKB_SOURCE_* stuff from structures.md is now actual function args. felt
good seeing my notes turn into code.

NOTE for when i use high_level: my hello_world Cargo.toml has default-features=false and only
allocator+dummy-atomic, so ckb-types/high_level isnt even compiled in yet. gotta add the
"ckb-types" feature when i start reading tx data for real (secret_lock will need it).

---

## THE GOTCHAS (this is the real content of the week)

### 1. default features want a C compiler
ckb-std default `libc` feature compiles C with riscv gcc/clang. i dont have it -> build dies
"program not found". fix:
  ckb-std = { version="0.16", default-features=false, features=["allocator","dummy-atomic"] }
now its pure rust, links with rust-lld, no gcc needed. lost time to this first.

### 2. MemWriteOnFreezedPage  <- the nasty one
built fine, then EXPLODED at runtime with this. took me ages.
reason: ckb-vm FREEZES executable pages. without a linker script my heap/.bss ended up on the same
memory page as the code. so the moment the allocator tried to write -> write to a frozen page ->
abort.
fix = the ckb-std.ld linker script. it uses PHDRS to put code and data in SEPARATE segments:
  code PT_LOAD FLAGS(5)  /* R + E */   read+execute
  data PT_LOAD FLAGS(6)  /* R + W */   read+write
plus `. = ALIGN(0x1000)` to page-align the boundary so they never share a page.
so that boring .ld file i kept asking about = the actual fix for this. lesson learned, its not
optional boilerplate.

### 3. debug! prints NOTHING in release
ckb_std::debug! is gated behind debug_assertions. in --release it compiles to literally nothing. i
was convinced my thing was broken because no output. it wasnt.
rule: deploy the RELEASE binary (small, silent), but when you want to SEE output use a DEBUG build.
and a live node never prints debug! anyway, thats debugger/test only.
(if you really want it in release: RUSTFLAGS="--cfg debug_assertions" cargo build --release)

---

## running it (two ways)

### off-chain, ckb-debugger
cargo install ckb-debugger  (needed gcc + protoc on PATH on windows, annoying)
ckb-debugger --bin target/.../debug/hello_world
->  Script log: Hello, World!
    Run result: 0
    All cycles: 18677
cycles = the gas-ish meter from last week. 18k for a hello world.

### on a real local devnet
set up a ckb node devnet, then:
1. deploy the binary as a CODE CELL (the elf goes in the cell's data). its data_hash becomes the
   code_hash other cells point at.
2. build a tx whose output has a TYPE SCRIPT pointing at that code cell, + a cell_dep to the code
   cell so the vm can find it.
3. sign, send. node runs my script during verification.
   return 0 -> committed (landed at block 0x443)
   return 1 -> TransactionFailedToVerify ... error code 1. the chain literally refused it.

this is the payoff: deploy code once, point at it by hash, the node executes it. exactly the
"script is a pointer not the code" thing from week2 but now i FELT it. on eth id deploy a contract
and call its address. here the binary lives in one cell and my cell just carries a hash + a dep.

---

## tooling rabbit hole

made hello_world, then made a 2nd project by hand-copying every file. dumb. so i tried to automate:
- offckb create -> gives a whole JS/TS dApp. wrong, i want pure rust scripts.
- the rust way = ckb-script-templates via cargo-generate.
- first try `cargo generate --path . --name x` -> made an EMPTY folder. turns out cargo-generate
  raw-copies the path AND tries to substitute every file, it ignores .gitignore, so it choked on
  the whole target/ dir of build junk and bailed.
- fix = a clean template dir with no target/ in it + {{project-name}}/{{crate_name}} placeholders.
  put it in code/ckb-template/. now: cargo generate --path ./code/ckb-template --name foo. works,
  renames properly, builds.
note: {{project-name}} keeps dashes, {{crate_name}} is the snake_case version. crate names cant
have dashes so main.rs uses crate_name.

---

## still fuzzy / TODO next week

* write secret_lock for real — lock checks a hash preimage from the witness. unlock w/ right
  secret, reject wrong one.
* need to actually USE load_witness + add the ckb-types feature to read it.
* write a type script (tiny UDT, supply conserved) — and FINALLY place a WitnessArgs by hand, the
  lock vs input_type vs output_type slots still havent clicked from reading alone.
* Type ID upgradeable deploy then freeze to data hash (carried over from last week, didnt get to).
* CCC playground — still unfinished, dropped it this week for the hands-on stuff. lower prio now.
* revisit: why exactly does the type-hash impostor attack work, still not 100%.
