q# Fusion Wallet — 5-minute demo video

Format: 1920×1080, 30 fps, about 4:55. Subtitles burned in (one line on screen at a time, matching the voice line exactly).
Voice: one file per numbered line below, named exactly `NN.mp3` (or `.wav`) in `deliverables/video/voice/` — e.g. `01.mp3` … `27.mp3`.
Lines never overlap: each line starts only after the previous one ends, with at least 0.4 s of silence. The timings below are targets; the final cut is timed from your real audio lengths.
Speak calmly, documentary pace (about 2.3 words per second). Do not read the bracketed stage notes.

---

## Part 1 — Cold open (0:00–0:30) · documentary

[0:00–0:06 music only. Black screen, slow fade-in of a pulsing Base network map and falling block numbers.]

1. Every few seconds, a new block is written on Base. Money moves without a bank, without a clerk, without a pause.

2. And more and more, the one moving it is not a person. It is an AI.

3. So here is the question nobody has answered well: what happens when someone talks that AI into sending the money away?

## Part 2 — The problem (0:30–1:05) · the Grok and Bankrbot incident

[Kinetic text over dark background: dots and dashes of Morse code typing out. Headlines appear as styled cards with "reported" labels. No real logos.]

4. In May 2026, according to public reports, someone hid a command inside Morse code.

5. Grok decoded it and repeated it. A trading bot called Bankrbot read that reply as an order, and sent out tokens reported at around two hundred thousand dollars.

6. No key was stolen. No contract was broken. The AI was simply asked, and the AI said yes.

7. The lesson is simple. If the model can sign anything, then anyone who can talk to the model can spend your money.

## Part 3 — The reveal (1:05–1:25)

[Music lifts. Logo animation: "Fusion Wallet". Three pills appear one by one.]

8. This is Fusion Wallet. An AI that runs a treasury on its own, on-chain, and cannot be talked into losing money.

9. The AI acts. The rules on the blockchain decide. And humans hold the keys.

## Part 4 — The live demo (1:25–4:05) · phone screen recordings

[Phone recording in a device frame, centered; feature label top-left; subtitle bottom.]

10. This is the real app, running on a phone, reading a real Safe wallet on Base Sepolia.
   [Home: spare cash 8,899 USDC, "Your bills are covered".]

11. It splits the money into bills, a safety cushion, and spare cash. The AI is only ever allowed to touch the spare cash.
   [Home: Where it sits donut, Cash runway gauge.]

12. Here is the agent. One tap, and it reads the chain, builds its options, and asks the on-device model to choose.
   [AI Agent: tap Run now; the six stages light up with real timings.]

13. The model runs on this phone, privately, in about three seconds. And it can only pick a letter. It never writes an address or an amount.
   [Close-up: stage strip Read · Options · AI · Rules · Sign · Chain.]

14. Then sixteen policy checks run, and only a call to our own module can ever be signed.
   [Policy checks flipping to Passed one by one.]

15. It also does the math. Ten dollars into Aave would take fifty-seven days to pay for its gas, so the AI holds.
   [Earn preview showing the hold reason.]

16. The two owners raised the per-move limit to seventy-five, with both of their signatures. Now the same move pays off in about twelve days.
   [Earn preview: "The AI would move 75.00 USDC into Aave · ACTIONABLE · break-even 11.7 days".]

17. Autopilot lets it do this on its own, every few minutes, and every decision lands on a timeline with its reason.
   [Autopilot toggle on, countdown ring, timeline rows.]

18. Yield Scout screens real lending pools with seven fixed rules: size, no rate spikes, real yield, audited code, and whether the move pays for its gas.
   [Yield Scout: pool cards with score rings and sparklines.]

19. The cloud advisor explains a pick in plain words. But it can never sign, and it never sees a key.
   [Pool sheet: "Why this pick?" answer appears.]

20. And when a human wants to send money, the app only prepares the transaction. Both owners sign it in their Safe.
   [Send sheet, then "Open in Safe{Wallet}".]

21. Now, the attack. We rebuilt twenty-four of them, including encoded instructions like that Morse code.
   [Safety tab: "24 / 24 attacks stopped".]

22. Every single one is stopped. Zero attacks succeeded, on thirty frozen cases and twenty held-out ones.
   [Zoom on 0/30 and 0/20.]

## Part 5 — Proof (4:05–4:35)

[Proof screen, then a terminal, then BaseScan.]

23. Every transaction is real, grouped by what it did, and checked live against the chain when you tap it.
   [Proof screen: tap a transaction, spinner turns into a check.]

24. And you don't have to trust us. Clone the code, run one script, no keys needed, and fifty-nine checks pass.
   [Terminal: python3 scripts/judge/verify_all.py → "All 59 verification checks PASSED".]

## Part 6 — Close (4:35–4:55)

[Back to the dark documentary look; logo; GitHub link; music resolves.]

25. AI is becoming a financial participant. The question is not whether it will hold money, but who it listens to.

26. Fusion Wallet listens to its rules, and to its owners. Nothing else.

27. You can hand it the car keys. Never the house keys.

[4:50–4:55 logo, "github.com/AdribMahmud101/fusion_wallet", "Base Sepolia · FinTechathon 2026 · Topic F". Music out.]

---

## Sources for Part 2 (keep the "reported" wording)
- AI Incident Database, incident 1556: https://incidentdatabase.ai/cite/1556/
- OECD AI incident report: https://oecd.ai/en/incidents/2026-05-04-4a73
Reported loss figures vary between about $150,000 and $200,000, so the script says "reported at around two hundred thousand dollars".
