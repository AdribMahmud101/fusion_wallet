# fusion-proxy

A small Cloudflare Worker (`fusion-proxy.adribmahmud.workers.dev`) that keeps the DeepInfra key out of the Android app.

- `POST /v1/openai/chat/completions`: forwards to DeepInfra. Only two models are allowed, output is capped at 700 tokens, the body is capped at 32 KB, and requests are rate limited (20 per minute per client, 120 per minute overall).
- `GET /yields/base-usdc`: the DeFiLlama pool list (about 11 MB) filtered to USDC lending pools on Base and cut to a few KB. Cached for 10 minutes.
- Nothing else is served. The key is a Worker secret (`wrangler secret put DEEPINFRA_API_KEY`) and is never in this repo or the APK.

The app only uses this for the cloud advisor and the Yield Scout feed. A user who stores their own key in Settings goes straight to DeepInfra instead.
