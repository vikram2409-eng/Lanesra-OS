#!/usr/bin/env python3
"""Apply website modernization (fresh build)."""
import subprocess
import os

REPO = "."

for patch in ["site_fresh.patch"]:
    p = os.path.join(REPO, "scripts", patch)
    if os.path.exists(p):
        subprocess.run(["git", "apply", p], cwd=REPO, check=True)
        print(f"Applied {patch}")

print("Done")
