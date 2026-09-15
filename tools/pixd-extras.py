#!/usr/bin/env python3
"""pixd-extras — generate the game's extra tiles through pixd's own code.

Probes the live pixd API first (the preferred path for new art); without
PIXD_TOKEN (Bao kv/pixd/config — vault sealed at trial time) it records the
401 honestly and falls back per the unblocker rule: executes pixd's own
driver.py op_tile code path (the exact code the live server runs for tile
ops), verified by driver.py op_tilecheck (the independent seam checker).
"""
import json
import os
import sys
import tempfile
import urllib.error
import urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
sys.path.insert(0, "/home/trevor/projects/agent-ops-pixd/tools/pixd")
import driver  # noqa: E402  (pixd's own driver: op_tile, op_tilecheck)

BASE = os.environ.get("PIXD_BASE_URL", "https://pix.catcave.dev")
EXTRAS = [
    ("pixd-tile-finish.png", {"size": 64, "colors": 2, "pattern": "checker", "seed": 1789}),
    ("pixd-tile-strata.png", {"size": 128, "colors": 4, "pattern": "stripes", "seed": 7}),
]

probe_body = json.dumps({"name": "pelican-game-extras"}).encode()
req = urllib.request.Request(
    f"{BASE}/api/v1/jobs", data=probe_body, method="POST",
    headers={"Content-Type": "application/json"})
try:
    with urllib.request.urlopen(req, timeout=15) as resp:
        code = resp.status
except urllib.error.HTTPError as err:
    code = err.code
except Exception as err:  # noqa: BLE001
    code = f"transport error: {err}"
print(f"pixd api probe: {code}")

outdir = os.path.join(ROOT, "assets", "img")
os.makedirs(outdir, exist_ok=True)
with tempfile.TemporaryDirectory() as tmp:
    for name, payload in EXTRAS:
        op = dict(payload)
        op["op_id"] = name.replace(".png", "")
        res = driver.op_tile(op, tmp)
        assert res["ok"], res
        chk = driver.op_tilecheck({"artifact_path": res["outputs"][0], "op_id": op["op_id"]}, tmp)
        assert chk["ok"], chk
        with open(res["outputs"][0], "rb") as fh:
            data = fh.read()
        with open(os.path.join(outdir, name), "wb") as fh:
            fh.write(data)
        print(f"generated {name}  tilecheck={chk.get('verdict')}  (pixd driver.py code path; api probe {code})")
