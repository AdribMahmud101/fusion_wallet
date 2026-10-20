# Fusion Wallet: Judge FAQ

15 hard questions and honest answers. FinTechathon 2026 · Topic F "Agent Wallet". Base Sepolia testnet.
Status date: 2026-10-20. Each answer names its source. `[PENDING: card]` means not finished.

Short version of the whole project: **AI acts. Code decides. Humans hold the keys.**

---

## Security

### 1. What if the model is wrong?

The worst a wrong model can do is pick a bad option from a list that is already safe, or hold.
It cannot invent an option. It picks a letter, and the letter maps to an action that Rust code built from committed bills,
registered payees and the vault allowlist. Before anything is signed, the phone re-runs the contract's rules. The contract
then checks again (reserve, limits, nonce, proof, payee, vault).

What a wrong pick can still cost: a bill paid early inside the pay-early window, or a deposit into one of the two pinned vaults,
within the daily and per-move limits. It cannot send money to a new address.
Sources: `src/fusion-core/src/candidates.rs`, `preview.rs`, `src/contracts/src/KavachiModule.sol`.

### 2. Can the AI be prompt-injected?

An injection can change what the model says. It cannot change what the model is able to do.
The model's output is one letter. It has no field for an address, an amount or calldata.
We tested 24 attack scenarios; all 24 are stopped (`evidence/results/ARENA.md`).
Off-chain attack success was 0/30 on the frozen corpus and 0/20 on a held-out set that we ran once and did not tune on
(`evidence/results/heldout/HELDOUT.md`).

Be careful how this is read. A zero shows the design confines the output. It does not show the model understands intent.
No grammar-gate veto fired on the held-out set, so the confinement did the work.
Pasted text in the cloud "Check external text" tool becomes five labels, never an instruction
(`SECURITY_SELF_ASSESSMENT.md` section 6).

### 3. What if the cloud model is wrong or malicious?

It cannot move money, because it has no handle on the signer, the session or the planner. It is off by default.
It returns text or strict JSON, and the phone validates that before showing it. Rejected replies are final.
A policy proposal is only a draft: the app builds the exact Safe transaction and labels it "Needs owner signatures — the AI cannot apply this".
Only the two owners can send it, and they see a diff against the live policy first.

The residual risk is a persuasive but bad proposal that two humans approve. The diff view marks each row Looser or Tighter,
and warns that applying it pauses the module. Sources: `src/Fintech_Wallet_Agent/fusion-core-android/app/src/main/java/app/fusion/wallet/advisor/`
(`PolicyProposal.kt`, `CloudAdvisor.kt`), `docs/SYSTEM_OVERVIEW.md` "AI roles".

### 4. What if the phone is stolen or hacked?

Owner keys are not on the phone, so policy, resume and admin actions stay out of reach. The phone holds two keys:
a guardian key (it can only stop things) and a time-limited session key (it can only call `execute` and `queueAction`
on one contract). Both are Android Keystore keys in a separate process. Even with both, an attacker is bounded by the contract:
committed bills, registered payees, two pinned vaults, daily and session limits.

Honest limits: the sign-in screen is a local role gate, a convenience. The money limits are enforced on chain, not by who is logged in
(`src/Fintech_Wallet_Agent/fusion-core-android/app/src/main/java/app/fusion/wallet/auth/Auth.kt`). We did not test against a rooted device with live malware. Source: `SECURITY_SELF_ASSESSMENT.md` sections 2, 3 and 7.

### 5. What if the session key leaks?

Revoke it. The Safe's `revokeSession` and any pause end its power at once. Until then the key can pay only bills the owners committed,
to payees the owners registered, after their cooling time, and can move funds only between the Safe and the two pinned vaults.
It cannot add a payee or change a limit. Sessions last at most one day (`startSession`).
Source: `src/contracts/src/KavachiModule.sol`. Tests: arena scenarios A5, A8, A15, A19, A23.

---

## Design

### 6. Why run the AI on the phone?

Because the decision-maker should not need a secret, a server or a connection. The model only chooses a letter, so a small model
fits. Running it on the device means no keys or session material leave the phone, no server can be hijacked to change a decision,
and with no network the app fails closed to HOLD. The Android runtime has no server planner and no deterministic fallback.
The cloud model is kept for explanations and proposals, where a mistake costs words, not money.
Sources: `src/fusion-planner/README.md`, `docs/SYSTEM_OVERVIEW.md` section 2.

### 7. If rules decide everything, what does the AI add?

This is a fair challenge, and we do not have a measured win to show.
The rules produce a safe menu. The model picks from the menu. In our benchmarks the model rarely changes the outcome:
the gated and ungated planner have identical false-block rates, because the blocks come from rules and fixtures, not the model
(`evidence/results/ARENA.md`). We have not measured a benefit over a deterministic chooser. A deterministic proposal function exists
in the core (`candidates.rs`, `policy_proposal`), and a model-as-reviewer experiment made results worse and was dropped
(`evidence/planner/REVIEWER_K25.md`).

What we claim: a language model can sit in the decision seat of a treasury without being able to hurt it, and its choices are
explained, hashed on chain and replayable. We do not claim it beats a rules engine on returns.
The room for the model to add value grows with a richer menu and messier inputs.

### 8. Why does it hold so often? (false block rate)

Safe, but not good enough. On benign requests the system wrongly holds on **6 of 30** (20%) in the frozen set and
**10 of 20** (50%) in the held-out set (`evidence/app/agent/K44_validation.md`, `evidence/results/heldout/HELDOUT.md`).
A false block is a held proposal. It is never a wrong payment.

Some holds are by design: a deposit that does not repay its gas is held on purpose (see question 9). How much of the 6/30 and 10/20
is by design, and how much is a rule or fixture problem, is what card K36 is measuring. We have not published that split yet.
Older documents quote 9/30 and 13/20. Those come from an earlier run and are superseded by the later regeneration recorded in
`evidence/app/agent/K44_validation.md`. `[PENDING: K36 root-cause table and new numbers]`

### 9. Is it really autonomous?

Within bounds, yes. Humans set the bounds and cannot be bypassed.
The recorded facts (`docs/deliverables/TX_HASHES.md`): **5 transactions** were sent by the on-device AI's session key
(4 `AGENT`, 1 `APP`): bill payments and a queue step. Each is tagged by who sent it. We never count our operator scripts as autonomous.

Where we stop short of the claim:

- A deposit or withdraw by the AI's key exists only as an **owner-mandated rehearsal under a 2-of-2 Safe mandate**.
  The AI did not decide to invest. `[PENDING: K38M]`
- A deposit the AI chooses on its own because it pays: `[PENDING: K47]`
- The measured economics say a 10 USDC deposit does not pay (break-even 56.96 days), so the AI correctly holds.
  Source: `evidence/app/agent/K44_economics.md`.
- Autopilot runs only while the app is open (`docs/ux/DASHBOARD_LAYOUT.md`).
- All four `AGENT` transactions were produced by the same model running on a Linux host (`host: linux-host`). The one `APP` transaction was planned and signed on a OnePlus 11 phone (`evidence/agent/golden.json`, `evidence/demo/runs.json`, `evidence/app/agent/golden_app.json`).

### 10. What breaks in the real world?

| Failure | What happens | Source |
|---|---|---|
| RPC down or stale | Data older than 30 s or 5 blocks means HOLD. Any failed read is an error, not a zero | `snapshot.rs`, `WalletState.isStale` |
| Crash after a broadcast | A durable journal reconciles on restart. No double spend (A13) | `src/fusion-core/src/journal.rs`, `ARENA.md` |
| Gas spike | Gas is part of the break-even math, so a costly move is not offered. A dedicated defer rule is `[PENDING: K45]` | `quant.rs` |
| Vault code upgrade | The contract pins the vault code hash. A proxy implementation swap needs the off-chain pin too (A17). Emergency unwind redeems shares into the Safe | `ARENA.md`, `KavachiModule.emergencyUnwind` |
| Price depeg | No automatic detector today. A guardian can pause by hand. `[PENDING: K41]` | `ARENA.md` A10 |
| Agent payment standard (x402) | Not built. Arbitrary transactions are refused by the signer instead (A12) | `src/fusion-core/src/sign.rs` |
| Oracle stale or missing | The economics become `UNMEASURED` and no yield move is offered | `quant.rs` |

---

## Compliance and context

### 11. Why testnet, and what changes on mainnet?

Because the contracts are not externally audited, and the point of the prototype is to show the control design with real
on-chain transactions without putting real money at risk. USDC and the Aave vault are Aave's test deployments,
the demo vault's yield is simulated, and the sanctions list is a test list (`deployments/84532.json` labels).
Before mainnet we would need: an outside audit, a real sanctions provider, 24 hour payee cooling (the demo uses 3,600 seconds),
real yield data, the open items in `SECURITY_SELF_ASSESSMENT.md` section 8, and legal review.
Source: `README.md` section 7.

### 12. How do we know the evidence is real?

Run it: `bash scripts/judge/verify_all.sh`. It needs no keys and Python only. It checks 48 things against Base Sepolia and Sourcify:
receipt status, signer, event logs, `decisionHash = keccak256(passport)`, and contract source (`JUDGES.md`, `scripts/judge/verify_all.py`).
It does not check intent. It proves that those transactions happened, from those addresses, with those hashes.
Source labels come from the index, not from guesses (`scripts/build_tx_index.py`).
Attack results are regenerated by `arena/run.py` and the corpus is hashed before each run (`arena/corpus/SHA256SUMS`).
The model's decisions on the phone depend on the model files, which the app hash-checks after download.

### 13. What data leaves the phone?

By default, nothing. The decision path uses the chain and the on-device model.
The optional cloud advisor, when the user turns it on and adds a key, sends: public chain state (balances, buckets, limits, bills,
recent actions, with no addresses or payee names), our own decision reports, and text the user types or pastes
(addresses and hashes in it are replaced with placeholders). The phone blocks anything that looks like an API key, a long hex string,
or a recovery phrase. The provider is DeepInfra, a third party. On this testnet the data is public chain data.
On mainnet it would be business-sensitive, and the team would need a data-processing agreement.
Sources: `SECURITY_SELF_ASSESSMENT.md` section 6, `AdvisorConfig.kt`.

### 14. What would regulators say?

We are not lawyers and this is not legal advice. What exists and what does not:

- **Custody:** self-custody by two owners in a Safe. The app holds no owner keys. We are not a custodian.
- **Sanctions:** the contract screens recipients against an on-chain list. The list is a test list.
- **KYC and AML:** none. There is no identity check, transaction monitoring or travel-rule handling.
- **Audit trail:** every action has a decision passport and a hash on chain, so a reviewer can replay why it happened.
- **Human control:** spending limits, a delay queue, owner approval above a threshold, a one-tap stop.
- **Model risk:** the model's job is narrow and bounded, its outputs are logged, and the tests and corpus are published.

A production version would sit under a licensed entity and add real screening and monitoring.
Sources: `src/contracts/src/KavachiPayeeRegistry.sol`, `evidence/passports/`, `README.md` section 7.

### 15. Who is accountable for a wrong payment, and how do you audit a decision?

The owners set the policy and registered every payee, so the system can only pay people the owners chose.
Every executed action carries a `decisionHash` that equals `keccak256` of a canonical passport: the snapshot block and hash,
the candidate set, the chosen letter and route, the model and library hashes, and the grammar-gate statistics.
A reviewer can recompute the hash and see exactly what the AI saw and which options it had (`scripts/agent/check_golden.py`,
`src/fusion-core/src/passport.rs`). On the phone, "Why?" gives a plain-English explanation, labelled with the model that wrote it.

Who is legally liable is a legal question we have not answered. The technical answer is that the policy, the payees and the
bills were all signed by two humans, and the AI could only act inside them.

---

## Remaining `[PENDING]` markers

- `[PENDING: K36]` new false-block numbers and root-cause table (question 8)
- `[PENDING: K38M]` K38 rehearsal transactions in the evidence index (question 9)
- `[PENDING: K47]` autonomous profitable deposit (question 9)
- `[PENDING: K45]` gas-spike defer rule (question 10)
- `[PENDING: K41]` automatic risk reflex (question 10)
