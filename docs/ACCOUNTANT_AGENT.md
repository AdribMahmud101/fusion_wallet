# Accountant Agent — design & data-egress guardrails

Status: implemented in `repos/Fintech_Wallet_Agent` (branch `fusion-v3-integration`), package `app.fusion.wallet.accountant`. See `CONTEXT.md` for progress.

## What it is
A **read-only** analyst. It looks at the treasury (cash, Aave/ERC-4626 lending positions and aTokens, obligations, 30-day cash flow) and explains it: liquidity, reserve coverage, concentration, yield, runway. It can run two ways:

| Part | Where | Data | Always on? |
|---|---|---|---|
| **LocalAccountant** | on device, pure Kotlin | exact figures | yes |
| **Cloud Accountant** (DeepInfra) | cloud LLM | coarse brief only | **no — opt-in, default OFF** |

It has **no authority**. Its output is advisory text. It never reaches the candidate generator, policy engine, signer or contract. Fund movement stays: on-device LFM picks a candidate ID → Rust preview → signer re-check → contract limits.

> **Spec change, deliberately made by the project owner (2026-10-17):** the original spec forbade any private context reaching a cloud LLM (only a fixed public-topic enum). This feature relaxes that to a *sanitized, coarse, read-only brief* behind an explicit consent and an egress guard. `LOCAL_ONLY` remains the default. The spec's `PRIV-02/03/04` and `CLOUD-0x` acceptance intents still apply to this path (canary isolation, local handover on financial incident, no cloud fallback for private work, cloud text is untrusted).

## Embedded database (Room / SQLite, on device)
Money is always a **base-10 string of raw integer units** (no floats). Chain facts are snapshots tagged with the block they were read at.

| Entity | Purpose |
|---|---|
| `account` | treasury / agent / guardian accounts, operating buffer B |
| `asset` | native, ERC-20, **aToken**, ERC-4626 share; decimals; `isStable` |
| `balance_snapshot` | wallet balance per asset per block |
| `lending_position` | **Aave V3 / ERC-4626 supply**: aToken balance (`suppliedRaw`), scaled balance, liquidity index (ray), `supplyApyBps`, borrowed, block |
| `category` | private label + safe `FlowCategory` enum |
| `obligation` | owner-approved liabilities (R), due dates, status |
| `ledger_entry` | payments/income/deposits/withdrawals with the spec's tx lifecycle states |
| `bucket_snapshot` | derived free / reserved / deployed / sweepable / shortfall history |
| `insight` | Accountant output (LOCAL or CLOUD), linked to the egress row that produced it |
| `egress_log` | **audit of every cloud attempt**, incl. blocked ones; stores the exact payload sent + SHA-256 |
| `privacy_settings` | cloud on/off, consent version, byte cap, daily call cap, model |

Buckets follow spec §3: `free = C−R`, `reserved = min(C,R)`, `deployed = V`, `sweepable = max(C−R−B,0)`, `shortfall = max(R−C,0)`; reference state C=5,000 R=1,500 B=500 V=5,000 → 3,500 / 1,500 / 5,000 / 3,000 (unit-tested).

## Data classification — what may leave the phone
| Class | Examples | To cloud? |
|---|---|---|
| **SECRET** | private keys, seeds, API keys, raw signed txs | **Never.** Not stored in Room at all. API key is AES-GCM wrapped by the Android Keystore. |
| **PRIVATE** | addresses, tx/block/invoice hashes, counterparties, memos, category labels, exact balances/amounts, absolute dates, non-stable balances | **Never.** |
| **COARSE** | totals to 2 significant figures, integer percentages, counts, APY (bps), relative due windows, fixed-enum categories | Yes — only inside the brief |
| **PUBLIC** | topic enum, protocol names, stable-asset symbols | Yes |

### "How much data" — hard limits
- One JSON **brief** (`fw.accountant.brief.v1`), ≤ **4 KB**, ≤ 8 lending lines, ≤ 5 obligation windows, ≤ 12 flow lines.
- Obligations only as **aggregates per due window** (`OVERDUE | 0-7d | 8-14d | 15-30d | 31d+`): count + 2-sig-fig total. No per-invoice rows.
- Flows only by **fixed category enum × direction**, last 30 days.
- Free-text question: ≤ 200 chars.
- ≤ **20 cloud calls / 24 h** (configurable).
- Consent version is bumped whenever the brief's fields change → user must re-consent.

### Read-only guarantees
1. The agent is constructed with `AccountantReadModel` (read) and `AccountantAudit` (append its own insights + audit rows). It has no handle on balances, obligations, ledger, policy, planner or signer.
2. The `cloud/` and `agent/` code has no Room/DAO/database references — enforced by a unit test that scans the source.
3. The cloud model gets **no tools**. Its reply is parsed against a fixed schema (`summary`, `findings[]`, `followUps[]`), length-capped, and rejected on any link or address. It is rendered as plain text, never executed.

### Egress guard (fail-closed, `EgressGuard`)
A payload is sent only if **all** hold: valid JSON ≤ cap; every key on the fixed schema and every value matches its type/enum/range; **amounts are exactly ≤ 2 significant figures** (so an exact balance can't pass even if the builder is buggy); no address / hash / key / URL / e-mail / seed-phrase shape anywhere; no long digit runs in free text; none of the device's **known private strings** (addresses, counterparties, memos, hashes pulled from the DB) appear in it. Block reasons are rule IDs only — they never echo the content, and a blocked payload is not stored.

## DeepInfra specifics (from docs.deepinfra.com)
- `POST https://api.deepinfra.com/v1/openai/chat/completions`, `Authorization: Bearer <key>`, OpenAI-compatible; we use `response_format: {type: json_object}`, `temperature 0.2`, `max_tokens 700`, no streaming.
- **Host is pinned** to `api.deepinfra.com` over HTTPS; redirects are not followed.
- DeepInfra states it does not store prompts/outputs on disk and does not log request content, **except Google and Anthropic models**, which are subject to those providers' retention. The client **refuses `google/` and `anthropic/` model IDs**.
- Rate limiting is HTTP 429 (no schedule documented) → up to 2 retries with 1 s / 2 s back-off; 5xx retried the same way; 401/403 not retried.
- Default model `meta-llama/Llama-3.3-70B-Instruct-Turbo` (present in DeepInfra's `/v1/openai/models` list on 2026-10-17); configurable.
- Residual risk: the **coarse** brief still reveals approximate size/shape of the business to DeepInfra and the network path. That is the consented trade-off.

## API key handling
- Release: user pastes the key in Settings → stored AES-GCM under a non-exportable Keystore key.
- Debug builds only: `DEEPINFRA_API_KEY=...` in the **gitignored** `fusion-core-android/local.properties` is baked into `BuildConfig` as a fallback.
- **Never commit a key** (this repo is public). A key shared in chat/terminal should be rotated after testing.

## Tests (JVM, no device)
Buckets vs. spec reference state; coarse rounding; brief contents; guard (unknown keys, exact balances, addresses/hashes/URLs/e-mail/keys/seed phrases, long digits, canary deny-list, size cap, rule IDs don't leak); client (model refusal, no key, request shape, 429/5xx retry, auth, network, bad body, host pinning); agent (off by default, consent version, happy path audit, private counterparty blocked pre-network, daily cap, provider failure → local, malicious reply rejected, no data, no client); structural read-only check.
