#!/usr/bin/env python3
import subprocess, os
REPO = "."
p = os.path.join(REPO, "scripts", "site_mobilefix.patch")
subprocess.run(["git", "apply", p], cwd=REPO, check=True)
print("Applied mobile fix")
