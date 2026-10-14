#!/usr/bin/env python3
"""Apply website modernization patches."""
import subprocess
import os

REPO = "."

# Patches are uploaded alongside this script
for patch in ["site_appjs.patch", "site_text.patch"]:
    p = os.path.join(REPO, "scripts", patch)
    if os.path.exists(p):
        subprocess.run(["git", "apply", p], cwd=REPO, check=True)
        print(f"Applied {patch}")
    else:
        print(f"Missing {patch}")

# New files are uploaded directly
print("Done")
