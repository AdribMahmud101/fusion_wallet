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
