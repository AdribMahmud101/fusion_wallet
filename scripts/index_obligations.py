#!/usr/bin/env python3
"""Publish every committed bill set under its own Merkle root.

The app resolves bill details for whatever root is committed on-chain right
now. Owners commit new roots over time (setup, the agent golden path, every
live demo run), so a hand-kept list of file paths always goes stale. This
copies each obligations file to evidence/obligations/by-root/<root>.json so a
client can fetch exactly the set that matches the chain, by root alone.
"""
import json
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
OUT = ROOT / "evidence" / "obligations" / "by-root"


def roots(path):
    try:
        data = json.loads(path.read_text())
    except (ValueError, OSError):
        return None
    root = data.get("root") if isinstance(data, dict) else None
    if isinstance(root, str) and root.startswith("0x") and len(root) == 66:
        return root.lower(), data
    return None


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    written = 0
    for path in sorted((ROOT / "evidence").rglob("*obligations*.json")):
        if OUT in path.parents:
            continue
        found = roots(path)
        if not found:
            continue
        root, data = found
        target = OUT / f"{root}.json"
        body = json.dumps(data, indent=2, sort_keys=True) + "\n"
        if not target.exists() or target.read_text() != body:
            target.write_text(body)
            written += 1
            print(f"{root}  <- {path.relative_to(ROOT)}")
    print(f"{written} written, {len(list(OUT.glob('*.json')))} roots indexed", file=sys.stderr)


if __name__ == "__main__":
    main()
