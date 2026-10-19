#!/usr/bin/env python3
"""Compare every callable/event/error ABI component with the Phase-1 freeze."""
import json
from pathlib import Path

root = Path(__file__).resolve().parents[1]
frozen = json.loads((root / "../../spec/abi/IKavachiModule.json").read_text())
actual = json.loads((root / "out/KavachiModule.sol/KavachiModule.json").read_text())["abi"]
actual = [entry for entry in actual if entry["type"] != "constructor"]
# Parameter names and tuple components, mutability, error fields and indexing all stay frozen.
canonical = lambda abi: sorted(json.dumps(entry, sort_keys=True) for entry in abi)
assert canonical(actual) == canonical(frozen), "Module ABI diverges from freeze"
print(f"ABI parity: {len(frozen)} entries identical (concrete constructor excluded)")
