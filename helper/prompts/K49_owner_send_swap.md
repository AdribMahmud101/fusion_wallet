# K49 — Owner Send / Swap with the AI boundary (no owner keys in the app, nothing dummy)

Android: submodule `src/Fintech_Wallet_Agent/fusion-core-android` (commit there, then the pointer). Read `docs/WIN_PLAN.md`
invariants (owner keys never in the app; AI only via module), the Home hero (Receive / Earn buttons) and WalletState buckets.

Why: judges expect a human escape hatch. The owner keeps sovereign Send/Swap; the AI never gets them.

1. Home hero gets "Send" next to Receive / Earn (keep the row clean; 3 buttons).
2. Send sheet (owner path):
   - Recipient (checksummed 0x address; validate; show a short warning if it is not a known payee), amount in USDC.
   - Reserve invariant check from live buckets: amount <= spare cash -> green "Bills and safety cushion stay covered";
     amount > spare cash -> red "This would dip into reserved bills/cushion by X USDC" and an explicit "I understand" toggle before
     the payload is shown.
   - Output: the exact Safe transaction the owners must sign: to = USDC token, value 0, data = ERC-20 transfer(recipient, amountRaw)
     (encode with the existing ABI helpers; unit-test the calldata), operation CALL. Show it as copyable fields + a QR of the JSON.
   - Button "Open in Safe{Wallet}" -> https://app.safe.global/home?safe=basesep:<SAFE_ADDRESS> (use the address from WalletState).
   - Label: "Owner-signed 2-of-2 · this app holds no owner keys · the AI cannot send".
3. Swap button -> sheet: "Swaps are owner-signed in Safe{Wallet}. The AI never swaps." + Open in Safe{Wallet}. If you cannot confirm
   Safe{Wallet} swaps support Base Sepolia, say "Swaps are not available on the Base Sepolia test network" (honest; no fake quote).
4. "Who can do what" card (Safety screen, and a compact line under the hero): Owner (2-of-2): Send · Swap · policy · resume.
   Guardian: pause · cancel queued. AI agent: Deposit · Withdraw · Rebalance · Pay verified bills · Hold — within on-chain limits.
   Values for limits come from the existing allowance formatter.
5. The Send/Swap code must not import or call any signer, session, planner or AgentRuntime class (add a unit test or lint-style test
   that checks the package dependencies if practical).
6. Tests + `flock /root/.gradle/fusion-build.lock ./gradlew testDebugUnitTest assembleDebug` (`ANDROID_HOME=/opt/android-sdk
   GRADLE_USER_HOME=/root/.gradle`). Update `docs/ux/DASHBOARD_LAYOUT.md`. Commit.
