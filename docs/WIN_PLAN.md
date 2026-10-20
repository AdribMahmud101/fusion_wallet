# Final win plan (2026-10-20)

Deadline 15:59 UTC. Feature freeze 12:30 UTC. Public APK and pushed repo 13:00 UTC. The team makes PDFs and video 13:00–15:59 UTC.

## Scoring map
| Criterion | Weight | What judges must see | Cards |
|---|---|---|---|
| Task completion | 40% | AI autonomously executes all three Topic F operations on-chain: asset management, transaction execution, risk control. Real hashes, AGENT/APP provenance. | K38 A, K39 B, K41 D |
| Security & compliance | 30% | On-chain enforcement, permission tiers, 24/24 attacks stopped, honest FBR, signer allowlist, self-assessment | K35 (done), K36, K42 |
| Innovation & interaction | 30% | Two-brain design: on-device AI executes within on-chain limits; cloud AI advises, explains, drafts policy that owners sign. Autopilot timeline. Polished UI. | K39 B, K40 C, K37, K43 |
| On-chain bonus | 0–5 | Safe module, decision hash, obligations Merkle root, x402 refusal, reproducible verification | all |

## Design rules (every card)
- The on-device planner is the only thing that chooses executable actions. The cloud model never signs, never picks executable actions, never sees keys.
- Every number on screen comes from chain or evidence. No dummy UI. No invented constants.
- Every new on-chain tx is indexed with honest `source` (AGENT, APP, OWNER, SCRIPT) and passes `scripts/judge/verify_all.py`.
- Android builds go through one lock: `flock /root/.gradle/fusion-build.lock ./gradlew ...` (edge RAM).
- Secrets: DeepInfra key lives in `/root/.secrets/deepinfra.env` (edge) only. Never in git, APK, logs, or evidence.

## Cards
| Card | Account | Owns (files) | Goal |
|---|---|---|---|
| K38 A | codex_3 | agent runtime execution path, runbook scripts, evidence/app | AI executes Aave deposit ≤10 USDC and withdraw on Base Sepolia; explain/fix HOLD with idle cash |
| K39 B | codex_1 | new `agent/autopilot/*`, AI Agent screen | Autopilot: periodic ticks, execute or hold, timeline |
| K40 C | antigravity_2 (after K34) | new `advisor/*`, Accountant screen, Settings key entry | Cloud advisor: explanations, Q&A, goal-to-policy proposals |
| K41 D | antigravity_1 (after K36) | new `agent/risk/*`, guardian auto path | AI risk reflex: detect anomaly, guardian pause on Sepolia |
| K43 | agy_1 (after K37) | new `ui/proof/*`, onboarding watch-only path | Judge proof screen and watch-only mode |
| K42 | after freeze | docs only | Security self-assessment, tech doc, demo script, deck outline (markdown sources) |

Supervisor (Claude): verify each card, merge, Tecno screen check, verify_all, APK release `app-latest`.
