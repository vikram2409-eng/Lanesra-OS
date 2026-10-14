#!/usr/bin/env python3
import subprocess, os
REPO = "."
for patch in ["site_nav_css.patch", "site_appjs_small.patch"]:
    p = os.path.join(REPO, "scripts", patch)
    if os.path.exists(p):
        subprocess.run(["git", "apply", p], cwd=REPO, check=True)
        print(f"Applied {patch}")
print("Done")
