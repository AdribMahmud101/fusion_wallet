# K45 — Dual-path event router: heartbeat, cognitive trigger, reflex interrupt

Read `docs/WIN_PLAN.md`, `docs/Perception & Ingestion.md` section 1.D, and the merged K39 (autopilot) code; K41's
`agent/risk/ReflexRouter.kt` if merged. Android submodule as usual.

Unify when the agent thinks:
1. Heartbeat: K39 Autopilot interval tick (keep).
2. Cognitive trigger: wake the planner early (between heartbeats, from each Home refresh) when deterministic metrics say it is
   worth it: idle spare cash above a threshold with an ACTIONABLE break-even (K44 if merged), or bucket drift > 5% from target,
   or a new obligation due soon.
3. Reflex interrupt: K41 reflex signals bypass the planner entirely (zero LLM) and go to the guardian path when Auto-protect is on.
4. Gas-spike defer: if baseFeePerGas exceeds a threshold, non-urgent moves are deferred (HOLD "gas spike, deferred") and retried
   on the next tick; bill payments due now are not deferred.
5. Fail-closed: any input unreadable → HOLD with reason; never execute on partial data.
6. AI Agent screen: show which path woke the agent on each timeline row (Heartbeat / Trigger: <reason> / Reflex: <signal>).
7. Unit tests; `flock /root/.gradle/fusion-build.lock ./gradlew testDebugUnitTest assembleDebug`
   (`ANDROID_HOME=/opt/android-sdk GRADLE_USER_HOME=/root/.gradle`). Update `docs/ux/DASHBOARD_LAYOUT.md` and the flow
   diagram in `docs/SYSTEM_OVERVIEW.md`.
8. Timeline rows must say WHY: a HOLD row currently reads "H · Hold / Hold · no transaction requested". Show the real reason from
   the decision report (e.g. "Nothing worth moving: deposit breaks even in 70 days" from K44, "Bill not due until 24 Oct",
   "Gas spike, deferred"). Same for the Home agent tile "Last decision".

## Addendum: AI Agent screen cleanup (found on device)
9. Duplicates: the screen has the K39 Autopilot card (Run now, Last decision, timeline) AND the older "Agent controls" card
   (Dry run toggle, "Run agent now") plus a separate "Latest decision" card. Merge into ONE flow: Autopilot card on top is the
   only place to run the agent (Run now, with the dry-run/live mode shown and switchable only when live is possible); the decision
   detail (planner, timings, candidates, policy checks, passport hash, tx) opens from a timeline row as a detail sheet. Remove the
   old controls and the standalone Latest decision card. Keep "How it decides", Daily limit, Decision mix, Queued actions.
10. Human-readable everything: format timings ("Load 1.45 s · Inference 2.88 s"; omit fields that were not recorded instead of
   "not recorded ms"); translate check messages to plain language with real units, e.g. "nonce_ok=true, epoch_ok=true…" → "Nonce,
   epoch, deadline and decision hash all valid"; "assets=0, hard_max=2500000000" → "Amount 0 USDC · max per action 2,500 USDC";
   "daily_used=2161000000/4000000000, session_used=8000000/7998000000" → "Used today 2,161 / 4,000 USDC · this session 8 / 7,998 USDC".
   Do this in the display layer (core output and evidence stay unchanged).
11. Allowance consistency bug: that check reports daily_used 2,161 / 4,000 USDC and hard_max 2,500 USDC per action, while the
   "Daily limit" tile shows 0% and Home shows "4,000 of 4,000 left · up to 10.00 per move". Read `src/contracts` (daily window /
   epoch reset, hard max, any per-move cap) and the policy reader; find which number is wrong or stale, fix it at the source, and make
   every screen (Home, AI Agent, Safety) show the same numbers from one formatter. Reuse K37's allowance labels if merged. If the
   daily counter has reset since `daily_used` was recorded, say "resets at <time>" instead of showing a stale number.
12. No endless scroll. The AI Agent screen must fit in about 2 phone screens (1080x2436) at most:
   - Top: compact Autopilot hero (status, toggle, interval, Run now, next tick countdown) in one card.
   - Then a 2x2 bento of small summary tiles: Daily limit (ring), Decision mix (mini bar), Queue (count), How it decides (opens sheet).
   - Then the timeline: show the latest 5 rows only, with "See all ticks" opening a full-screen list or bottom sheet.
   - Everything long (decision detail, candidates, policy checks, passport, how it decides) lives in sheets or collapsed sections,
     never inline. Policy checks: summary line "16 passed · 0 failed" that expands.
   - Use segmented tabs inside the screen if needed (e.g. "Autopilot | Decisions | Limits") rather than stacking.
   Apply the same rule to Home and Safety if they exceed ~2.5 screens: collapse secondary cards, move detail to sheets.
   Take before/after screenshots on the emulator or describe heights in the commit message.
