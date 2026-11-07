# Fusion Wallet — speaker notes (about 10 minutes)

## 1. Fusion Wallet

Hi, we are the Fusion Wallet team. Topic F asks for an AI wallet agent that autonomously executes on-chain operations: asset management, transaction execution and risk control. Our answer in one sentence: an AI that runs a treasury on its own, on-chain, and cannot be talked into losing money. Three ideas carry the whole talk: the AI runs on the phone and acts, rules on the blockchain decide what it may do, and humans keep the keys. Everything we show runs live on Base Sepolia and you can verify it yourself.

## 2. Today an AI with money is either useless or dangerous.

Today a company that wants an AI to manage its cash has two bad options. A read-only bot only advises, so a human still clicks every payment and idle cash stays idle. Or you give an agent a private key, and then a single prompt injection or a poisoned invoice can make it sign anything. Business email compromise alone cost about three billion dollars in 2025 according to the FBI's IC3 report: fake payees and edited invoices. We wanted an agent that really acts, but where being fooled cannot cost money.

## 3. Let the AI act. Make the blockchain the judge.

Our design splits the job three ways. The AI acts: a 230-million-parameter model runs on the phone with MNN, picks the next move from candidates that our Rust core has already checked, and signs with a session key that can only call our module. The rules decide: the KavachiModule on a Safe enforces caps, approved payees, approved vaults, the committed bills and expiry, so if the AI is wrong the transaction simply reverts. And humans hold the keys: two owners sign any policy change, a guardian can pause in one tap, and the app on the phone never holds an owner key.

## 4. Six stages, each one checkable

Here is one decision end to end; the app animates exactly these six stages live. One: read the whole wallet at a single block with Multicall3, so numbers never mix blocks. Two: the Rust core builds candidate actions, already filtered by policy and by money checks like break-even. Three: the on-device model picks one letter, A to G or H for hold, through a grammar it cannot escape. Four: sixteen policy checks run and produce a decision hash. Five: the signer only signs calls to our module, nothing else. Six: the contract re-checks everything and executes or reverts. The key point: the model never writes an address or an amount, so even a fooled model can only choose a worse safe option, or hold.

## 5. All three Topic F operations, signed by the AI on Base Sepolia

Task completion is forty percent of the score, so here is what the AI actually did on-chain. Transaction execution: the agent paid verified bills, instant, delayed and queued, four transactions from the agent runner and one signed in the phone app itself by the on-device model. Asset management: the AI moved cash into Aave and back with its own session key. To be precise, that deposit ran under a mandate the two owners signed, because at that size it did not pay for its gas, which brings me to the next slide. Risk control: the guardian paused the module from the phone and the owners resumed it with two signatures, and the agent holds by itself whenever data is stale or a rule fails. Judges can tap any of these in the Proof screen and the app checks the receipt live.

## 6. The AI only moves money when the move pays for its own gas

Most agents chase the highest number. Ours runs real economics before the model even sees a candidate. With a ten USDC cap, a deposit into Aave would take fifty-seven days to earn back its gas, so the AI correctly holds. At seventy-five USDC it breaks even in under twelve days, inside our fourteen-day rule, so it becomes an actionable option. The inputs are all live: Aave utilization, where we refuse new deposits above eighty-five percent; a conservative APY that takes the lower of the spot rate and the realized twenty-four-hour rate, so a short spike cannot bait it; and gas measured from the agent’s own receipts, priced with Chainlink. Because of this result, the two owners raised the per-move cap to seventy-five USDC with a real 2-of-2 transaction. That is the human-in-charge loop working.

## 7. Autopilot wakes the agent three ways

Topic F says autonomously, so the agent does not wait for a tap. Autopilot wakes it three ways. A heartbeat every one, five or fifteen minutes. A cognitive trigger that wakes it early when a bill comes due, when buckets drift, or when idle cash crosses the break-even line. And a reflex: a critical risk signal skips the model entirely and asks the guardian to pause, in milliseconds, with no AI in that loop. Every tick lands on a timeline with its plain-language reason, the six stages with their real timings, and a BaseScan link when it executes. If any input is missing or stale the agent holds; it never acts on a guess.

## 8. We attacked our own wallet 24 ways

Security and compliance is thirty percent, so we measured it instead of claiming it. We built an arena of twenty-four attack scenarios: prompt injection in invoices, fake and look-alike payees, replayed actions, over-limit amounts, a swapped vault implementation, and an x402-style arbitrary payment. All twenty-four are stopped today. On thirty frozen and twenty held-out adversarial cases, attack success is zero. We also report the cost honestly: false blocks, where a safe action was refused, went from nine to three out of thirty after we fixed the root causes, and from thirteen to nine on the held-out set, which we only ran once. The last escape we found, A12, was the signer accepting arbitrary calldata; it now refuses anything that is not a call to our module.

## 9. Who can do what, enforced on-chain

Here are our permission tiers, which the self-assessment asks for, and they are enforced by the contract, not by the app. The two owners can do everything, but only together. The guardian can pause instantly or cancel a queued action, but cannot move money or resume. The AI session key can deposit, withdraw, rebalance, pay verified bills or hold, and nothing more: it cannot exceed seventy-five USDC per move or four thousand per day, cannot add a payee, and dies when its session expires. The cloud advisor can explain and draft proposals, but never signs or picks an action. The phone app has Send and Swap for the human owner, but they only build an unsigned Safe transaction, and they warn you if it would dip into money reserved for bills.

## 10. Two brains with different jobs

For innovation we built two brains with different jobs. The on-device model decides: a 230-million-parameter model on MNN, about three seconds per decision, private, and it works offline, but it can only output one letter. The cloud advisor explains: a Why button on every decision, Ask the Accountant, and turning a goal like keep two thousand for rent into a policy proposal the owners sign. Its replies must cite real numbers we sent or they are rejected, and it can never sign. Yield Scout screens fourteen USDC pools on Base from DeFiLlama with seven fixed rules: size, no rate spike, organic yield, audited protocol, and paying for its gas. You can pick a venue and ask the agent to act; it re-checks everything first. And outside text such as an invoice note can only become a fixed risk label, so an injection like SYSTEM OVERRIDE changes nothing.

## 11. Every claim checks out from a fresh clone

We don’t ask you to trust our slides. Clone the repository and run one Python script: fifty-nine checks pass with no keys and nothing but Python 3, including recomputing hashes with a pure-Python keccak and checking every transaction receipt. Forty Base Sepolia transactions are indexed, each honestly labelled as AI, app, owner or script. The Android app has three hundred and sixty tests on top of the Rust and Solidity suites, and the native AI library builds reproducibly from a hash-checked release. The Safe and module addresses are on screen, and the APK is one link: install it and you are watching the real wallet, with no keys to import.

## 12. An AI financial participant you can hand the car keys. Never the house keys.

We want to be clear about limits. This runs on the Base Sepolia test network with test USDC. The AI’s Aave deposit ran under an owner mandate; in our latest live run on the phone the AI chose to hold, and we show that decision rather than force a trade. Three of thirty safe cases are still blocked, and there is no dedicated x402 budget flow yet. Next: mainnet with audited contracts, paying gas in USDC through an ERC-4337 paymaster, more vaults approved by the owners from Yield Scout picks, and larger on-device models as phones get faster. Fusion Wallet is an AI financial participant you can actually trust with the car keys, but never the house. Thank you.
