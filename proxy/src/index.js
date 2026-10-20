// fusion-proxy: keeps the DeepInfra key off the phone and serves a small, filtered yield feed.
// Only two routes exist, only two models are allowed, output is capped, and everything is rate limited.
const MODELS = new Set(["deepseek-ai/DeepSeek-V3.2", "Qwen/Qwen3-235B-A22B-Instruct-2507"]);
const MAX_BODY = 32 * 1024;
const MAX_TOKENS = 700;
const JSON_H = { "content-type": "application/json", "cache-control": "no-store" };

const reply = (obj, status = 200, extra = {}) => new Response(JSON.stringify(obj), { status, headers: { ...JSON_H, ...extra } });

async function limited(env, req) {
  const ip = req.headers.get("cf-connecting-ip") || "unknown";
  const [a, b] = await Promise.all([env.PER_CLIENT.limit({ key: ip }), env.GLOBAL.limit({ key: "all" })]);
  return !(a.success && b.success);
}

async function chat(req, env) {
  const len = Number(req.headers.get("content-length") || 0);
  if (len > MAX_BODY) return reply({ error: "too large" }, 413);
  let body;
  try { body = await req.json(); } catch { return reply({ error: "bad json" }, 400); }
  if (!MODELS.has(body.model)) return reply({ error: "model not allowed" }, 403);
  if (!Array.isArray(body.messages) || body.messages.length > 4) return reply({ error: "bad messages" }, 400);
  const out = {
    model: body.model, messages: body.messages, stream: false,
    max_tokens: Math.min(Number(body.max_tokens) || MAX_TOKENS, MAX_TOKENS),
    temperature: Math.min(Math.max(Number(body.temperature) || 0.2, 0), 0.7),
  };
  if (body.response_format && body.response_format.type === "json_object") out.response_format = { type: "json_object" };
  if (JSON.stringify(out).length > MAX_BODY) return reply({ error: "too large" }, 413);
  const up = await fetch("https://api.deepinfra.com/v1/openai/chat/completions", {
    method: "POST",
    headers: { "content-type": "application/json", authorization: `Bearer ${env.DEEPINFRA_API_KEY}` },
    body: JSON.stringify(out),
  });
  const text = await up.text();
  return new Response(text, { status: up.status, headers: JSON_H });
}

const num = (v) => (typeof v === "number" && isFinite(v) ? v : null);

async function yields(ctx) {
  const cache = caches.default;
  const key = new Request("https://fusion-proxy.internal/yields-base-usdc-v1");
  const hit = await cache.match(key);
  if (hit) return new Response(hit.body, { status: 200, headers: { ...JSON_H, "x-cache": "hit" } });

  const r = await fetch("https://yields.llama.fi/pools", { headers: { accept: "application/json" } });
  if (!r.ok) return reply({ error: "upstream", status: r.status }, 502);
  const all = (await r.json()).data || [];
  const base = all
    .filter((p) => p.chain === "Base" && p.symbol === "USDC" && p.stablecoin === true && p.exposure === "single" && num(p.tvlUsd) > 1_000_000)
    .sort((x, y) => y.tvlUsd - x.tvlUsd)
    .slice(0, 14);

  const sparks = await Promise.all(base.slice(0, 8).map(async (p) => {
    try {
      const c = await fetch(`https://yields.llama.fi/chart/${p.pool}`);
      const pts = (await c.json()).data || [];
      return pts.slice(-30).map((d) => num(d.apy)).filter((v) => v !== null);
    } catch { return []; }
  }));

  const pools = base.map((p, i) => ({
    id: p.pool, project: p.project, symbol: p.symbol, tvlUsd: num(p.tvlUsd), apy: num(p.apy),
    apyBase: num(p.apyBase), apyReward: num(p.apyReward), apyMean30d: num(p.apyMean30d),
    spark: sparks[i] || [],
  }));
  const body = JSON.stringify({ source: "DeFiLlama", fetchedAt: Math.floor(Date.now() / 1000), chain: "Base", pools });
  const res = new Response(body, { status: 200, headers: { ...JSON_H, "cache-control": "public, max-age=600" } });
  ctx.waitUntil(cache.put(key, res.clone()));
  return new Response(body, { status: 200, headers: { ...JSON_H, "x-cache": "miss" } });
}

export default {
  async fetch(req, env, ctx) {
    const url = new URL(req.url);
    if (url.pathname === "/health") return reply({ ok: true });
    const isChat = req.method === "POST" && url.pathname === "/v1/openai/chat/completions";
    const isYield = req.method === "GET" && url.pathname === "/yields/base-usdc";
    if (!isChat && !isYield) return reply({ error: "not found" }, 404);
    if (await limited(env, req)) return reply({ error: "rate limited" }, 429, { "retry-after": "30" });
    try { return isChat ? await chat(req, env) : await yields(ctx); }
    catch (e) { return reply({ error: "proxy failure" }, 502); }
  },
};
