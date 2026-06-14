# week2 :: cell + tx structure (my own words)

this week was mostly structure. week1 i just got the general idea, now i actually went thru each field one by one. writing it down cause i forget otherwise. also i kept comparing to btc/eth in my head the whole time, thats the only way half of it makes sense to me so im putting that here too.



---

## the Cell itself

smallest unit in ckb. everything is a cell. it looks like:

Cell { capacity, lock, type, data }

capacity = a number (how many bytes this cell is allowed to occupy)
lock = a Script, says who owns it
type = a Script, optional, says what RULES govern it
data = the actual bytes you store

> btc cousin: a btc output (CTxOut) is basically { nValue, scriptPubKey }. money + a lock. thats it. a ckb cell is that PLUS a data field PLUS an optional type script. so its a utxo that can also carry arbitrary data and arbitrary rules.
> eth: theres no "cell". eth has accounts that own a balance and contracts that own a big storage mapping. nothing is this little self contained box.

---

## capacity (this clicked finally)

capacity is doing TWO jobs at the same time and thats the trick that confused me for ages:
1. its how much ckb token the cell holds (the balance)
2. its the max number of bytes the cell is allowed to take up onchain

so holding 100 ckb literally means you have the right to occupy 100 bytes of the chain. money == space. same thing.

the validation rule i wrote down:
capacity_in_bytes >= len(capacity) + len(data) + len(type) + len(lock)

ie the cell has to be big enough to hold its own fields. and when you make outputs, occupied bytes cant exceed the capacity you assigned.

min cell = 61 ckbytes. even a empty cell still has to store capacity(8) + lock(32) + args(20) + hash_type(1) = 61 bytes. so you cant have a cell smaller then 61 ckb. worth remembering, you cant just send someone 5 ckb in a fresh cell, floor is 61. (got tripped up by this already)

> btc: nValue is JUST the money. it has nothing to do with size, a btc output doesnt pay rent for its bytes. ckb fused money and storage on purpose so the chain doesnt bloat for free.
> eth: storing data on eth costs gas ONCE (SSTORE) then it sits there forever and you never pay again, which is why eth state is bloated. ckb makes you lock capacity for as long as you occupy the space. thats the state rent idea from week1, and capacity is the mechanism.

---

## Script (the structure)

a Script is NOT code. its a pointer to code plus its arguments. shape:

Script { code_hash, hash_type, args }

code_hash = fingerprint of the actual binary (which lives in some other cell)
hash_type = how to interpret code_hash (data vs type, more below)
args = the arguments fed to that code (eg a pubkey hash)

important distinction i kept messing up: Script vs Code.
Script = the little data struct that POINTS at code.
Code = the actual RISC-V binary (ELF file) that runs in the VM.
a "code cell" = a cell whose data field is that binary.
the script never contains the code, it just references it by hash. like a function pointer.

> eth: an eth contract glues code + storage + address together in one thing. ckb rips them apart: the binary sits in one cell, your cell just holds a hash pointing at it, and args is your personal config. so 1 million cells share ONE binary and differ only by args.
> btc: scriptPubKey is the raw script bytes inlined into every output, no pointer, no sharing, no args concept. ckb is way more modular.

---

## Lock Script (who owns it)

the lock answers "who is allowed to spend this cell". typically code_hash points to a signature checker (like the default secp256k1 one) and args holds your pubkey hash. to spend, you drop a signature into the witness, the lock script reads sig + checks it against args, returns 0 if you're really the owner.

> btc: this IS scriptPubKey. the lock conditions on an output, you satisfy them to spend. ckb literally says lock is "similar to scriptPubKey in CTxOut". the diff is btc script is a fixed dumb stack language, ckb lock is a full risc-v program so the sig algo isnt hardcoded, you can write your own (schnorr, ed25519, whatever, even passkeys).
> eth: ownership is just msg.sender == some address, checked inside contract logic. there's no separate reusable "lock" object, every contract reinvents auth.

note to self: lock ONLY runs on inputs. makes sense, you only need a permission check when SPENDING something, not when receiving.

---

## Type Script (the rules)

optional. governs how a cell can transform. if an input cell and an output cell share the same type script, the change between them MUST obey that type script's rules. classic use = a token (UDT). the type script enforces "tokens out == tokens in, no printing money".

big one, writing in caps so i dont forget: TYPE SCRIPT RUNS ON BOTH INPUTS AND OUTPUTS. unlike lock which is inputs only. its because the rules has to check the before state and the after state. it even runs when the cell gets destroyed.

the split that makes ckb special:
lock = ownership (who)
type = application logic (what rules)
two totally separate jobs, separate scripts. thats literally the first class assets thing from week1, now i can see WHERE it lives in the structure. a buggy type script cant override your lock.

example i liked: alice has a cell, data = her token balance, type = the token's rules. to pay bob she spends her cell as input, makes an output cell with the SAME type script but bob's lock. if she doesnt send everything she also makes a change cell back to herself. very utxo, very btc-like, but with rules attached.

> eth: a token is one ERC20 contract holding balances[everyone] in its own storage. the contract = owner + rulebook + ledger all mashed into one. on ckb that's torn into per user cells (lock) + shared rules (type). hack the rules on eth and everyone's balance is gone, on ckb the rules cant touch your lock.
> btc: no equivalent at all, btc has no notion of "rules for state transition", it only has ownership locks. type script is the thing btc completely lacks.

---

## code_hash + hash_type (how the VM finds the code)

since the script only stores a hash, the VM has to go find the real binary. it looks through the transaction's cell_deps (the attached dependency cells) and matches by hash. HOW it matches depends on hash_type:

hash_type = data / data1 / data2  -> code_hash is the blake2b hash of the dep cell's raw DATA (the binary bytes). exact match. immutable. also pins the VM version (data=v0, data1=v1, data2=v2). frozen forever, fully reproducible, even across a future hard fork.

hash_type = type -> code_hash is the hash of the dep cell's TYPE SCRIPT instead. so any cell carrying that type script counts as a match. lets the code be UPGRADED: deploy a new binary under the same type script and everyone pointing at it follows to the new version. always runs on the latest VM.

so the tradeoff:
data hash = immutable, auditable, "i run exactly these bytes forever"
type hash = upgradeable, "i follow whatever the maintainer ships"

> eth: this is EXACTLY the immutable contract vs proxy/upgradeable contract debate. data hash = a plain immutable contract, the bytecode is set in stone. type hash = a proxy pattern where the address stays but the impl behind it can be swapped. same exact tradeoff, same exact trust question (do you trust whoever can push the upgrade).
> btc: nothing. script is frozen the instant it's in an output, zero upgrade story.

the danger with type hash: multiple cells can share the same type script but hold DIFFERENT code. so a bad actor could maybe slip in a malicious binary under the same hash. thats why type hash is advanced mode only and you protect it with a Type ID. (still not 100% sure i get the attack here, revisit)

Type ID = a special system script that makes a cell a singleton, only ONE live cell can ever exist with that type script hash. so the type hash points to exactly one unique cell, no impostors. it's enforced at protocol level. basically "this upgradeable pointer can only ever resolve to the one canonical code cell".

and it's not a permanent choice, you can take a type-ID deployed script, grab the current binary's data_hash, and redeploy as hash_type data to freeze it. switch from upgradeable to locked whenever you want.

---

## args / script_args

the args field = the input arguments handed to the script when it runs. convention: pubkey goes in args, signature goes in witnesses. but it's only convention, not enforced.

one gotcha the docs flagged: ckb does NOT use the normal unix argc/argv to pass args. the script has to pull args in via ckb syscalls inside the risc-v VM. so "args" isn't like a function parameter you just receive, you load it.

> eth: like the constructor/immutable args baked into a contract, except here it's per cell so the same code behaves differently per cell based on its args.

---

## Script Group Execution (why it's not run a million times)

if 5 inputs all use the same lock script, the VM does NOT run it 5 times. it GROUPS inputs by their script and runs each group once. 3 steps:

1. grouping: bucket inputs by their lock script. note two inputs can share the same code but have different args, those are still grouped per (code+args) script. the doc example focuses on group g1 = inputs 0 and 2.
2. code locating: find the code in cell_deps by the data hash, load that binary.
3. execution: run the binary from its entry function.

while running, the script reads tx data via syscalls, eg:
ckb_load_script(addr, len, offset) to read itself
ckb_load_witness(addr, len, offset, 0, CKB_SOURCE_INPUT) to read the first witness

first 3 args = where to put the data + how much. then a source: CKB_SOURCE_INPUT reads from the tx's inputs. there's also CKB_SOURCE_GROUP_INPUT which reads using the index inside the GROUP's virtual array (only the cells/witnesses belonging to this group). same for type scripts but with CKB_SOURCE_GROUP_OUTPUT since type scripts also see outputs.

so a script basically has these "load_*" syscalls as its only window into the world. it's a sandboxed risc-v program that can READ the tx but not touch anything else.

> eth: same flavor as EVM context opcodes, CALLDATALOAD / CALLER / SLOAD etc, the contract reads its environment through special opcodes. ckb just uses posix style syscalls instead because the VM is real risc-v, not a bespoke VM. grouping itself has no eth equivalent, eth you'd batch calls manually and each is its own full execution context.

---

## the Transaction (putting it together)

a tx destroys some cells and creates new ones. fields:

version       -> bump for forks
cell_deps     -> read only cells holding code/data the scripts need
header_deps   -> block headers a script wants to read
inputs        -> the cells being consumed (CellInput)
outputs       -> the new cells (just lock/type/capacity, no data)
outputs_data  -> the data for each output, kept in a parallel list
witnesses     -> signatures + proof data

> btc: vin[] + vout[]. ckb inputs/outputs map straight onto that. ckb just adds cell_deps + header_deps + witnesses + a separate data list. the skeleton is pure btc utxo though, "spend these, create these".
> eth: a tx is { to, value, data, nonce, gas, sig }, one sender poking one address. there's no list of things consumed and created, the effects are hidden inside contract storage changes. you cannot look at an eth tx and see exactly what state it touches. on ckb the tx declares EVERY cell it reads or writes up front, which is the whole reason ckb can parallelize and eth can't.

tx lifecycle states i noted (don't need all the detail, just the shape): Pending -> Confirming -> Confirmed, plus the unhappy paths Conflicting / Conflictive / Reverted / Abandoned. reverted is basically "was confirmed, chain reorg'd, back to pending". the generator (the wallet/app) has to hold pending txs locally and keep resending because nodes can drop them from the pool. don't reuse outputs of a pending tx unless you're fee bumping. RBF fee bumping has been default in ckb since v0.112.1.

> btc: this whole pending/confirmed/reorg/RBF model is lifted straight from btc mempool behavior. felt familiar.
> eth: eth has pending/confirmed too but reorg handling is mostly abstracted by the node, you think in nonces not in tracking-your-own-utxos.

---

## cell_deps + dep_type

cell_deps = the read only cells a tx pulls in so its scripts can find their code (or just read some shared data). each entry = CellDep { out_point, dep_type }.

out_point = { tx_hash, index }, ie "the Nth output of tx X". that's how you point at any cell anywhere.

dep_type:
code -> the dep cell IS the binary, load it directly.
dep_group -> the dep cell holds a LIST of out_points, ckb expands it and loads each as if you'd listed them all. just a compact bundle so you don't have to list 5 deps by hand (the secp256k1 default lock needs a couple deps, dep_group ships them as one).

> eth: closest analogy is importing a library / linking, except on eth a library is also a deployed contract you DELEGATECALL into. here you literally attach the code cell to your tx as a dependency. the cell_deps list makes a tx's full code dependencies explicit and visible, eth dependencies (which contracts get called) are discovered at runtime.
> btc: no deps, script is inlined, nothing to reference.

---

## header_deps

list of block header hashes a script is allowed to read during execution. lets a script peek at chain history (timestamps, epoch, etc). constraint: every referenced header must already be on chain (no uncle blocks) so that all nodes compute the same result. determinism matters, every node must agree.

> eth: like reading block.timestamp / block.number inside a contract, except here you must explicitly declare which headers up front. eth just lets you read the current block context implicitly. the explicit declaration is again about keeping validation deterministic + parallelizable.
> btc: btc script can't read block headers at all (well, nLockTime/nSequence aside), so this is more powerful than btc script.

---

## inputs + since

inputs = list of CellInput. each = { previous_output, since }.

previous_output = the out_point { tx_hash, index } of the cell you're spending. again pure utxo pointer.

since = an optional time lock on that ONE input. the input (and thus the whole tx) isn't valid until the chain passes the since condition. can be expressed as block number, epoch number (with fraction), or a timestamp. and because each input has its own since, different parties in a multi party tx can each gate their own piece. but if ANY input's since isn't met yet, the whole tx is rejected.

> btc: this is literally btc's nLockTime / nSequence (CSV/CLTV) timelocks. ckb says as much. used for delayed payments, multi party coordination, etc.
> eth: no native per input timelock, you'd write require(block.timestamp > x) inside a contract yourself. ckb bakes it into the input field.

---

## witnesses (the messy one)

witnesses = a list of byte blobs the tx creator provides so scripts pass. mainly: signatures. convention again: pubkey in args, signature in witnesses.

a tx WITHOUT witnesses = a "raw transaction", and the tx hash is computed from that raw form (witnesses excluded from the hash). BUT witness length still counts toward tx size, so it still costs fee. neat detail: that's why you can know the tx hash before signing, signing doesn't change the hash.

witness is just bytes, you can serialize whatever proof you want (molecule or custom). to stop different scripts fighting over the same witness, there are conventions. the main one:

WitnessArgs (a molecule table) with 3 optional fields:
lock          -> data for the Lock script
input_type    -> data for the Type script when checking inputs
output_type   -> data for the Type script when checking outputs

why 3 slots? because lock runs on inputs only, but type runs on inputs AND outputs, so type needs two separate slots. each script reads only its own field out of the witness, so multiple scripts can share witness index N without stepping on each other (lock reads .lock, type reads .input_type, etc). the WITNESS INDEX lines up with the script group's index in the virtual array, that ordering matters.

i traced thru their group_exec example, took me a while. its a tx swapping two type scripts and the groups came out as
  lock_script_1: inputs[0], inputs[1]
  type_script_1: inputs[0], outputs[1]
  type_script_2: inputs[1], outputs[0]
and the same witness[0]/witness[1] got read by multiple groups but each only pulled its own field. that's the whole point of WitnessArgs, avoid collisions. (consumed ~245k cycles in their test, cycles = ckb's gas-ish meter.)

CoBuild = a newer witness layout standard for multiple parties co building a tx offchain, meant to eventually replace WitnessArgs. still early, not finalized, not widely used. just parking the name.

> btc: big parallel here. btc segwit literally moved the signature OUT of the input (scriptSig) into a seperate witness area, which is basically what ckb does, signed payload stays clean and witness is excluded from the txid (segwit fixed malleability the same way ckb computes the hash from the raw tx). when i realised ckb witness == segwit witness it kinda all clicked.
> eth: one signature wraps the entire tx (v,r,s), always exactly one signer = the from account. ckb can have many inputs from many owners each bringing their own witness, so multi party / multi sig txs without needing a smart contract. that's a real structural advantage.

---

## outputs_data

the data for output cells lives in its OWN parallel list, not inside the output objects. outputs_data[i] = the data for outputs[i]. they split it out to keep script execution / VM handling simpler and leave room for future protocol optimizations.

> eth: contract storage is a mapping tucked inside the contract, not a flat parallel list hanging off the tx. ckb keeps cell data as plain bytes in the tx, visible and addressable.

---

## blocks etc (skim, less important for building)

a block = header + transactions + uncles + proposals.
first tx in a block = the cellbase (miner's reward tx), same idea as btc's coinbase tx.
header carries the PoW nonce + difficulty (compact_target) + merkle roots + the dao field. ckb's PoW algo is Eaglesong (not sha256), check: pow = eaglesong(pow_hash || nonce), must be <= target.
uncle blocks = two blocks mined ~same time, only one makes the chain, the loser can be referenced as an uncle (like eth's ommers / uncles). miners get partial credit.
proposals = a tx must be PROPOSED in an earlier block before it's COMMITTED. on mainnet the window is 2 <= c - p <= 10 blocks. a two step commit to fight selfish mining / front running.

> btc: cellbase == coinbase, PoW == PoW (different hash algo), nonce == nonce. very btc.
> eth: uncle/ommer blocks are an eth concept too (pre merge). the propose-then-commit two phase thing is more ckb specific though, btc/eth just commit directly.

---

## still fuzzy / TODO next week

* actually USE since, never set a timelock yet
* when would i ever need header_deps in a real script??
* deploy my own code cell, feel code vs dep_group difference by hand
* WitnessArgs lock vs input_type vs output_type slots still trip me, want to build a tx with a type script and place the witness myself
* finish the CCC Playground examples (got halfway)
* Type ID, want to deploy an upgradeable script behind one and then freeze it to data hash
