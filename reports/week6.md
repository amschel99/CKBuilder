# Week 6 progress
 Working on Filament for the Fibre hackathon 
 
 Filament allows anyone to accept Fiber payments in one API call.

- Got stablecoins working. Minted a test USD token (fUSD) on the devnet and paid with it over Fiber — invoice, channel, payment, all in the stablecoin, not just CKB.
- Made both demos use it: an $8.00 coffee checkout, and a $0.05 pay-per-call API (L402) where an agent pays to unlock the data.
- Wrote the Docker setup and the testnet/mainnet config templates so the same thing can run on real networks later (fUSD swaps for RUSD).


Still running on devnet. Next: get it onto testnet with a real funded wallet and a Fiber node.
