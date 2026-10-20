# K39 (Card B) — Autopilot: the agent acts on its own

Read `docs/WIN_PLAN.md` first. Android app: submodule `src/Fintech_Wallet_Agent/fusion-core-android` (branch
`fusion-v3-integration`; commit there, then the pointer). Read `.../agent/README.md` and `AgentRuntime.kt` first.

## Why
"Autonomously executes" is the Topic F definition. Today the runtime is manual ("This is a manual foreground runtime").

## Build
1. `agent/autopilot/` (new package, you own it): Autopilot controller. When ON, runs `AgentRuntime` ticks every N minutes
   (default 5; choices 1/5/15) while the app is in the foreground, plus a "Run now". Use a foreground service with a
   persistent notification if it is clean; otherwise in-process coroutine tied to app lifecycle. Respect: module paused, session
   expired, read-only tier, guardian stop, no network → tick records HOLD with reason, never crashes.
2. Execution mode: if the device has a session key and broadcast is available (existing signer + relay lease path), executed
   ticks broadcast; otherwise Autopilot runs in dry-run and says so plainly ("Preview only on this device: no session key").
   Never fake an execution.
3. AI Agent screen: Autopilot card at top: toggle, interval, status (next tick countdown, last decision), and a timeline list of
   ticks (time, decision letter + plain title, reason, checks passed/failed, tx hash link when executed). Persist the timeline
   (last 50) in app storage. Stop button = turn off immediately.
4. Home agent tile shows "Autopilot on · next in 3m" when on.
5. Unit tests for scheduling, guard conditions, timeline persistence. Build with
   `flock /root/.gradle/fusion-build.lock ./gradlew testDebugUnitTest assembleDebug` (`ANDROID_HOME=/opt/android-sdk
   GRADLE_USER_HOME=/root/.gradle`; native libs via `scripts/android_planner/build.sh` + `build_core.sh` from worktree root).
6. Update `docs/ux/DASHBOARD_LAYOUT.md`. Do not touch `agent/risk/*` or `advisor/*` (other cards own them).
