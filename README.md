# storage_contract

## Project Title
storage_contract

## Project Description
`storage_contract` is a self-storage unit rental dApp built on the Stellar network using the Soroban smart-contract platform. Today, the self-storage industry relies on paper contracts, point-of-sale systems, and email confirmations, which makes disputes over late fees, early termination, and renewal messy. This project replaces the rental agreement with an on-chain ledger: a facility operator lists a unit of a given size and monthly rate, a renter books it for a defined term, and the contract records late fees, early termination, and renewals so that both parties share a tamper-proof rental history. No real XLM moves on-chain in this MVP — the contract focuses on the verifiable rental state machine that any future payment-token integration can plug into.

## Project Vision
The long-term vision is to become the trustless rental ledger for the entire self-storage industry, and eventually adjacent real-world rental verticals such as parking spaces, equipment lockers, and cold-storage rooms. With the rental state on-chain, a follow-up payment-token contract can settle monthly rent and late fees automatically, a multi-sig arbitrator can resolve disputes without taking custody, and renters can prove their history to underwriters or future landlords. The end state: storage rentals that are programmable, auditable, and accessible from any wallet on Stellar.

## Key Features
- **Unit listing** — Facility operators call `list_unit` to register a unit by size (square meters) and monthly rate. Each `unit_id` can only be listed once.
- **Time-bounded booking** — Renters call `rent_unit` to book a listed unit for a `[start, end]` window. The unit flips to `available = false` until the rental is terminated or expires.
- **Late-fee enforcement** — Operators call `mark_late` to flag a rental as late and record the assessed `late_fee`. The fee is capped on-chain to prevent operator abuse.
- **Early termination** — Renters call `terminate` to end a rental early with an on-chain reason code. The contract returns any previously assessed late fee for off-chain settlement.
- **Renewal** — Renters call `renew` to extend the `end` timestamp of an active rental and clear any outstanding late fee in a single on-chain step.
- **Public status view** — Anyone can call `get_rental` to read the current state of a rental (`0` Active, `1` Late, `2` Terminated, `3` Expired) by `rental_id`.

## Contract

- **Network:** Stellar Testnet (Public)
- **Scope:** real_estate dApp — see `contracts/storage_contract/src/lib.rs` for the full storage_contract business logic.
- **Functions exposed:** see `Key Features` above and the `pub fn` list in `lib.rs`.
- **Contract ID:** `CDANIS45K7U4HDFYA4UE3EFRPRQFMXSWF3LZYOFUEQ4TXUAO46GQLLPM`
- **Explorer template:** `https://stellar.expert/explorer/testnet/tx/75379182813c2d23f0768f790c499e431de20eba5b9ffe8723d3901ad8f9eee1`

## Future Scope
- **Native-asset payments** — Pair `storage_contract` with a Stellar asset (USDC or a wrapped stablecoin) so that monthly rent and late fees are debited automatically when a rental crosses a billing boundary.
- **Dispute resolution** — Add a multi-sig arbitrator role that can slash or refund a late fee based on off-chain evidence, keeping custody with the contract.
- **Vertical expansion** — Parameterize the `Unit` struct so the same contract can handle parking spots, equipment, and cold-storage rentals.
- **Freighter frontend** — Build a small web UI that lets operators list units, lets renters browse and book, and displays rental status from the contract state.
- **Test coverage** — Add a full unit-test suite (`#[test]` in `lib.rs` or under `test/`) covering happy paths, double-booking rejection, late-fee caps, and unauthorized termination.

## Profile

- **Name:** <!-- Fill github name -->
- **Project:** `storage_contract` (real_estate)
- **Built with:** Soroban SDK 25, Rust, Stellar Testnet
