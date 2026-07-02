# Week 4 Progress



## What I worked on

Finished `secret_lock`, and spent the rest of the week settling on what I want to build as a real
open-source contribution to the ecosystem. It came straight out of my own last three weeks: dev
tooling for RGB++.

### secret_lock — done
- A lock that only unlocks if the spender puts the right secret in the witness. The script loads the
  preimage from the witness, hashes it, compares against the hash in the script `args`, returns 0
  only on a match.
- First script where I had to read the witness inside a running script and branch on it. Wrong
  secret gets the tx rejected, right secret unlocks — same accept/reject demo as hello_world.

## The idea

The thing I kept fighting these past weeks wasn't the protocol — it was the tooling. No test harness,
hand-rolling every project, `offckb` handing me a JS dApp when I wanted Rust, the freezed-page error
eating an afternoon. If writing a `hello_world` was that rough, anyone trying to build on RGB++
(which spans Bitcoin *and* CKB at once) is in far worse shape. There's no good local way to test the
thing the whole ecosystem is being told to build.

So that's what I want to build: **"Hardhat/Anvil for RGB++"** — one command that boots a regtest
`bitcoind` and a CKB devnet wired together, with funded accounts on both chains, an automated
issue → leap → transfer → leap-back demo, fixtures, snapshots, and assertion helpers. It's an
integration job, not a new protocol: fuse OffCKB's CKB devnet, a regtest `bitcoind` (BitLight
already has a usable compose), and `rgbpp-sdk` — plus the connective tissue nobody has built. The
connective tissue is the actual value:

- **A local/mock Bitcoin SPV relay** that feeds regtest block headers into the CKB devnet so
  `RgbppLock` and `BtcTimeLock` actually validate locally. Without this you can't exercise the
  binding offline at all — this is the core missing piece.
- **Fast-forwardable `BtcTimeLock`** so the ~6-block reorg-safety leap delay doesn't make every test
  take an hour. Make it a knob.
- **Reorg simulation** — regtest lets you invalidate blocks; assert that RGB++ state resolves
  correctly. That's the failure surface that's terrifying on mainnet and basically untested today.
- **Paymaster mocking** — stub the BTC-UTXO-subsidizes-CKB-cell flow and the exchange-rate logic.
- **An assertion library** — `expectAssetBoundTo(utxo)`, `expectLeapComplete()`, paired-tx checks
  across both chains.

### Why this is the right build
- It's directly on CKB's #1 priority — RGB++/BTCFi is where the whole ecosystem is pointed right now.
- It helps every other team instead of competing with them — I'm building the thing they all need,
  not fighting them for the same users.
- Test infra is how tools become standards. Own the test loop and you end up load-bearing the way
  Foundry and Anchor are — you don't win on users, you win by being the thing everyone's CI runs.
- And I'm not guessing the gap exists. I hit it myself for three weeks straight.

## Open questions (being honest with myself)
- **Is someone already building this?** It extends OffCKB more than it replaces it, so before I sink
  weeks into it I want to be sure I'm not redoing something that's already coming. Going to run the
  whole idea past Neon first and get his read on it.
- **Scope.** The mock SPV relay + reorg simulation are the hard, valuable core. I should build those
  as the proof first and not get lost in polish (snapshots, nice CLI) until the core works.
- **Maintenance.** Fusing three moving upstreams (OffCKB, bitcoind, rgbpp-sdk) means version drift is
  a real ongoing cost I'll have to keep up with.

## Next week
- Talk it through with Neon — get his read on the idea and whether something like this already
  exists.
- Stand up the dumb version: `bitcoind` regtest + CKB devnet running side by side with funded
  accounts on both, nothing fancy.
- Spike the hardest piece — feeding regtest headers into the devnet so an `RgbppLock` cell validates
  locally. If that works, the rest is reachable. If it doesn't, I need to know now.
