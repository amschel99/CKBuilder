## Builder Track Weekly Report — Week 1


### Courses Completed

- Studied the **CKB foundational theory**, covering:
  - **What CKB is:** a **Layer 1 blockchain** designed around the **blockchain trilemma** (decentralization, security, scalability) and its multilayered solution — an L1 focused on security + decentralization, with L2s built on top for scalability.
  - **Key facts:** Proof of Work consensus, **CKByte** native token (1 CKByte = 1 byte of on-chain storage, pays gas, stakeable in the **Nervos DAO**), and the **CKB-VM** built on **RISC-V** where smart contracts execute.
  - **CKB vs Bitcoin:** how Bitcoin's stack-based **Script** and **UTXO** model work, and how CKB generalizes the UTXO into the **Cell model**.
  - **The Cell Model in depth:** Cell structure (`capacity`, `lock`, optional `type`), Cell lifecycle (Live → Dead via Consumption), and what a transaction actually declares.
- Worked through the **Bitcoin Script vs CKB Lock Script** comparison using the same "add two numbers and check they equal 8" example — stack-based opcodes vs. arbitrary code on CKB-VM.

### Key Learnings

- Developed a detailed understanding of the **Cell model as a generalized UTXO**, including:
  - The role and structure of the **`lock` script** (who controls the Cell) vs. the **`type` script** (rules for how state may change)
  - How values are passed in the **witness** field and validated by scripts returning `0` (success) or non-`0` (failure)
  - Why a transaction is just a declaration: *consume these Live Cells → create these new Cells*, gated by every Lock Script and Type Script returning `0`
- Understood the **first-class assets** security property:
  - On Ethereum your tokens are an entry in a contract's mapping (`balances[you] = 1000`) — the contract owns them
  - On CKB your tokens live in **Cells locked by your Lock Script**; a buggy Type Script **cannot bypass your Lock**
  - The trade-off is **state rent**, implemented in practice as secondary issuance + Nervos DAO dilution
- Learned **flexible transaction fees** — any party (sender, receiver, or third party) can attach the CKBytes that pay the fee, as long as `sum(input capacities) ≥ sum(output capacities) + fee`
- Grasped the **three scalability properties** of the Cell model:
  - Off-chain computation, on-chain verification
  - Parallel execution across CPU cores (txs declare exactly which Cells they touch)
  - Batched operations (fee scales with size, not number of operations)

### Practical Progress

- Read and annotated a real **Cell JSON structure** (`capacity`, `lock.code_hash`, `lock.args`, `hash_type`, `type`)
- Traced a **Bitcoin Script execution** step-by-step on a stack (`OP_3 OP_5 OP_ADD <8> OP_EQUAL`)
- Rewrote the same logic as a **CKB lock script** in pseudocode, loading values from the witness and returning `0`/`1`
- Mapped the full **transaction validation flow**: inputs flip Live → Dead, outputs become new Live Cells once all scripts pass

### Environment

- Notes and diagrams maintained in Markdown with Mermaid (`notes/intro.md`)
- Conceptual groundwork in place for the CKB Cell model, transaction structure, and CKB-VM execution model — ready to move from theory into hands-on tooling next week
